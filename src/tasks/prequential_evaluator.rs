use crate::classifiers::Classifier;
use crate::core::instance_header::InstanceHeader;
use crate::evaluation::{LearningCurve, PerformanceEvaluator, Snapshot};
use crate::streams::Stream;
use crate::utils::system::current_rss_gb;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Instant;

pub struct PrequentialEvaluator {
    learner: Box<dyn Classifier>,
    stream: Box<dyn Stream>,
    evaluator: Box<dyn PerformanceEvaluator>,

    curve: LearningCurve,

    max_instances: Option<u64>,
    max_seconds: Option<u64>,
    sample_frequency: u64,
    mem_check_frequency: u64,

    processed: u64,
    start_time: Instant,
    last_sample_time: Instant,
    last_mem_sample: Instant,
    ram_hours: f64,

    progress_tx: Option<Sender<Snapshot>>,
}

impl PrequentialEvaluator {
    pub fn new(
        mut learner: Box<dyn Classifier>,
        stream: Box<dyn Stream>,
        evaluator: Box<dyn PerformanceEvaluator>,
        max_instances: Option<u64>,
        max_seconds: Option<u64>,
        sample_frequency: u64,
        mem_check_frequency: u64,
    ) -> Result<Self, Error> {
        if sample_frequency == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "sample_frequency must be > 0",
            ));
        }
        if mem_check_frequency == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "mem_check_frequency must be > 0",
            ));
        }

        let header = stream.header();
        let header_arc = Arc::new(InstanceHeader::new(
            header.relation_name().to_string(),
            header.attributes.clone(),
            header.class_index(),
        ));
        learner.set_model_context(Arc::clone(&header_arc));

        Ok(Self {
            learner,
            stream,
            evaluator,
            curve: LearningCurve::default(),
            max_instances,
            max_seconds,
            sample_frequency,
            mem_check_frequency,
            processed: 0,
            start_time: Instant::now(),
            last_sample_time: Instant::now(),
            last_mem_sample: Instant::now(),
            ram_hours: 0.0,
            progress_tx: None,
        })
    }
}

impl PrequentialEvaluator {
    pub fn with_progress(mut self, tx: Sender<Snapshot>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.start_time = Instant::now();
        self.last_sample_time = self.start_time;
        self.last_mem_sample = self.start_time;

        while self.stream.has_more_instances() {
            if let Some(n) = self.max_instances {
                if self.processed >= n {
                    break;
                }
            }
            if let Some(s) = self.max_seconds {
                if self.start_time.elapsed().as_secs() >= s {
                    break;
                }
            }
            let Some(instance) = self.stream.next_instance() else {
                break;
            };
            self.processed += 1;

            // TODO: Remove this
            if self.processed == 581012 {
                println!("last element");
            }

            let votes = self.learner.get_votes_for_instance(&*instance);

            self.evaluator.add_result(&*instance, votes);

            self.learner.train_on_instance(instance.as_ref());

            if self.processed % self.mem_check_frequency == 0 {
                self.bump_ram_hours();
            }
            if self.processed % self.sample_frequency == 0 {
                self.push_snapshot();
            }
        }

        self.push_snapshot();
        Ok(())
    }

    pub fn curve(&self) -> &LearningCurve {
        &self.curve
    }

    fn push_snapshot(&mut self) {
        use std::collections::BTreeMap;

        let secs = self.start_time.elapsed().as_secs_f64();
        let perf = self.evaluator.performance();

        let mut acc = f64::NAN;
        let mut kap = f64::NAN;
        let mut extras = BTreeMap::new();

        for m in perf {
            let key: &str = m.name.as_ref();
            match key {
                "accuracy" => acc = m.value,
                "kappa" => kap = m.value,
                _ => {
                    extras.insert(key.to_string(), m.value);
                }
            }
        }

        let snapshot = Snapshot {
            instances_seen: self.processed,
            accuracy: acc,
            kappa: kap,
            ram_hours: self.ram_hours,
            seconds: secs,
            extras,
        };

        if let Some(tx) = &self.progress_tx {
            let _ = tx.send(snapshot.clone());
        }

        self.curve.push(snapshot);
        self.last_sample_time = Instant::now();
    }

    fn bump_ram_hours(&mut self) {
        let now = Instant::now();
        let duration = now - self.last_mem_sample;
        let dt_h = duration.as_secs_f64() / 3600.0;
        self.last_mem_sample = now;

        let rss_gb = current_rss_gb().unwrap_or(0.0);
        self.ram_hours += rss_gb * dt_h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::{BasicClassificationEvaluator, BasicEstimator, PerformanceEvaluator};
    use crate::testing::{ClassifierNoneVotes, OracleClassifier, TrainSpyClassifier, VecStream};
    use std::io::ErrorKind;

    #[test]
    fn ctor_guards() {
        let s: Box<dyn Stream> =
            Box::new(VecStream::new((0..10).map(|i| (i % 2) as usize).collect()));
        let l: Box<dyn Classifier> = Box::new(OracleClassifier::default());
        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));
        let err = PrequentialEvaluator::new(l, s, e, None, None, 0, 5)
            .err()
            .unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);

        let s: Box<dyn Stream> =
            Box::new(VecStream::new((0..10).map(|i| (i % 2) as usize).collect()));
        let l: Box<dyn Classifier> = Box::new(OracleClassifier::default());
        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));
        let err = PrequentialEvaluator::new(l, s, e, None, None, 5, 0)
            .err()
            .unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn periodic_and_final_snapshots() {
        let s: Box<dyn Stream> =
            Box::new(VecStream::new((0..100).map(|i| (i % 2) as usize).collect()));
        let l: Box<dyn Classifier> = Box::new(OracleClassifier::default());
        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));

        let mut pq = PrequentialEvaluator::new(l, s, e, None, None, 10, 7).unwrap();
        pq.run().unwrap();

        assert_eq!(pq.curve().len(), 11);
        let last = pq.curve().latest().unwrap();
        assert_eq!(last.instances_seen, 100);
        assert!(last.accuracy > 0.9999);
        assert!(last.kappa.is_finite() && last.kappa > 0.99);
        assert!(last.ram_hours >= 0.0);
    }

    #[test]
    fn stops_at_max_instances() {
        let s: Box<dyn Stream> = Box::new(VecStream::new(
            (0..1000).map(|i| (i % 2) as usize).collect(),
        ));
        let l: Box<dyn Classifier> = Box::new(OracleClassifier::default());
        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));

        let mut pq = PrequentialEvaluator::new(l, s, e, Some(25), None, 5, 3).unwrap();
        pq.run().unwrap();

        assert_eq!(pq.curve().len(), 6);
        assert_eq!(pq.curve().latest().unwrap().instances_seen, 25);
        assert!(pq.curve().latest().unwrap().accuracy > 0.999);
    }

    #[test]
    fn stops_immediately_when_time_zero() {
        let s: Box<dyn Stream> =
            Box::new(VecStream::new((0..100).map(|i| (i % 2) as usize).collect()));
        let l: Box<dyn Classifier> = Box::new(OracleClassifier::default());
        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));

        let mut pq = PrequentialEvaluator::new(l, s, e, None, Some(0), 10, 10).unwrap();
        pq.run().unwrap();

        assert_eq!(pq.curve().len(), 1);
        let last = pq.curve().latest().unwrap();
        assert_eq!(last.instances_seen, 0);
        assert!(last.accuracy.is_nan());
        assert_eq!(last.kappa, 0.0);
    }

    #[test]
    fn snapshot_frequency_math() {
        let s: Box<dyn Stream> =
            Box::new(VecStream::new((0..12).map(|i| (i % 2) as usize).collect()));
        let l: Box<dyn Classifier> = Box::new(OracleClassifier::default());
        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));

        let mut pq = PrequentialEvaluator::new(l, s, e, None, None, 5, 1).unwrap();
        pq.run().unwrap();

        assert_eq!(pq.curve().len(), 3);
        assert_eq!(pq.curve().latest().unwrap().instances_seen, 12);
    }

    #[test]
    fn votes_none_keeps_metrics_nan_and_zero() {
        let s: Box<dyn Stream> =
            Box::new(VecStream::new((0..20).map(|i| (i % 2) as usize).collect()));
        let l: Box<dyn Classifier> = Box::new(ClassifierNoneVotes::default());
        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));

        let mut pq = PrequentialEvaluator::new(l, s, e, None, None, 10, 2).unwrap();
        pq.run().unwrap();

        let last = pq.curve().latest().unwrap();
        assert!(last.accuracy.is_nan());
        assert_eq!(last.kappa, 0.0);
    }

    #[test]
    fn train_called_once_per_instance() {
        let labels: Vec<usize> = (0..37).map(|i| (i % 2) as usize).collect();
        let s: Box<dyn Stream> = Box::new(VecStream::new(labels));

        let (spy_cls, handle) = TrainSpyClassifier::new();
        let l: Box<dyn Classifier> = Box::new(spy_cls);

        let e: Box<dyn PerformanceEvaluator> =
            Box::new(BasicClassificationEvaluator::<BasicEstimator>::new_with_default_flags(2));

        let mut pq = PrequentialEvaluator::new(l, s, e, None, None, 10, 4).unwrap();
        pq.run().unwrap();

        assert_eq!(handle.count(), 37);
    }
}
