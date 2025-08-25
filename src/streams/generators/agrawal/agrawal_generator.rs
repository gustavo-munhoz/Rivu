use crate::core::attributes::{AttributeRef, NominalAttribute, NumericAttribute};
use crate::core::instance_header::InstanceHeader;
use crate::core::instances::{DenseInstance, Instance};
use crate::streams::generators::agrawal::function::AgrawalFunction;
use crate::streams::generators::agrawal::rules::{RawAttrs, determine};
use crate::streams::stream::Stream;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

#[derive(Debug)]
pub struct AgrawalGenerator {
    seed: u64,
    rng: StdRng,
    function: AgrawalFunction,
    balance_classes: bool,
    next_class_should_be_zero: bool,
    perturb_fraction: f64,
    header: Arc<InstanceHeader>,
    max_instances: Option<usize>,
    produced: usize,
}

impl AgrawalGenerator {
    pub fn new(
        function: AgrawalFunction,
        balance_classes: bool,
        perturb_fraction: f64,
        max_instances: Option<usize>,
        seed: u64,
    ) -> Result<Self, Error> {
        if !(0.0..=1.0).contains(&perturb_fraction) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "perturb_fraction must be in 0.0..=1.0",
            ));
        }
        Ok(Self {
            seed,
            rng: StdRng::seed_from_u64(seed),
            function,
            balance_classes,
            next_class_should_be_zero: false,
            perturb_fraction,
            header: Arc::new(build_agrawal_header()),
            max_instances,
            produced: 0,
        })
    }

    pub fn new_with_id(
        function_id: u8,
        balance_classes: bool,
        perturb_fraction: f64,
        max_instances: Option<usize>,
        seed: u64,
    ) -> Result<Self, Error> {
        let function = AgrawalFunction::try_from(function_id)?;
        Self::new(
            function,
            balance_classes,
            perturb_fraction,
            max_instances,
            seed,
        )
    }

    fn sample_raw_attributes<R: Rng + ?Sized>(rng: &mut R) -> RawAttrs {
        let salary = rng.random_range(20_000.0..150_000.0);
        let commission = if salary < 75_000.0 {
            0.0
        } else {
            rng.random_range(10_000.0..75_000.0)
        };
        let age_i = rng.random_range(20..=80);
        let elevel_i = rng.random_range(0..=4);
        let car_i = rng.random_range(1..=20);
        let zipcode_i = rng.random_range(0..=8);
        let hvalue = if zipcode_i == 0 {
            0.0
        } else {
            let low = 50_000.0 * zipcode_i as f64;
            let high = 100_000.0 * zipcode_i as f64;
            low + rng.random::<f64>() * (high - low)
        };
        let hyears_i = rng.random_range(1..=30);
        let loan = rng.random_range(0.0..500_000.0);

        RawAttrs {
            salary,
            commission,
            age: age_i,
            elevel: elevel_i,
            car: car_i,
            zipcode: zipcode_i,
            hvalue,
            hyears: hyears_i,
            loan,
        }
    }

    fn maybe_perturb(&mut self, a: &mut RawAttrs) {
        if self.perturb_fraction <= 0.0 {
            return;
        }

        let rng = &mut self.rng;

        if rng.random::<f64>() >= self.perturb_fraction {
            return;
        }

        let mult = |rng: &mut StdRng, x: &mut f64| {
            let sign = if rng.random::<bool>() { 1.0 } else { -1.0 };
            let factor = 1.0 + sign * self.perturb_fraction;
            *x *= factor;
        };

        mult(rng, &mut a.salary);
        mult(rng, &mut a.commission);
        mult(rng, &mut a.hvalue);
        mult(rng, &mut a.loan);

        let perturb_i = |rng: &mut StdRng, v: &mut i32| {
            let fv = *v as f64;
            let sign = if rng.random::<bool>() { 1.0 } else { -1.0 };
            let factor = 1.0 + sign * self.perturb_fraction;
            let nv = (fv * factor).round();
            *v = nv.clamp(0.0, f64::from(i32::MAX)) as i32;
        };
        perturb_i(rng, &mut a.age);
        a.age = a.age.clamp(0, 120);

        perturb_i(rng, &mut a.hyears);
        a.hyears = a.hyears.clamp(0, 60);
    }

    fn determine_class(&self, a: &RawAttrs) -> i32 {
        determine(self.function.as_u8(), a) as i32
    }
}

impl Stream for AgrawalGenerator {
    fn header(&self) -> &InstanceHeader {
        &self.header
    }

    fn has_more_instances(&self) -> bool {
        self.max_instances.map_or(true, |m| self.produced < m)
    }

    fn next_instance(&mut self) -> Option<Box<dyn Instance>> {
        if !self.has_more_instances() {
            return None;
        }

        let mut group;
        let (mut attributes, mut ok);

        loop {
            attributes = Self::sample_raw_attributes(&mut self.rng);
            group = self.determine_class(&attributes);

            if !self.balance_classes {
                ok = true;
            } else {
                let want_zero = self.next_class_should_be_zero;
                ok = (want_zero && group == 0) || (!want_zero && group == 1);
                if ok {
                    self.next_class_should_be_zero = !self.next_class_should_be_zero;
                }
            }

            if ok {
                break;
            }
        }

        self.maybe_perturb(&mut attributes);

        let mut values = Vec::with_capacity(10);
        values.push(attributes.salary);
        values.push(attributes.commission);
        values.push(attributes.age as f64);
        values.push(attributes.elevel as f64);
        values.push(attributes.car as f64);
        values.push(attributes.zipcode as f64);
        values.push(attributes.hvalue);
        values.push(attributes.hyears as f64);
        values.push(attributes.loan);
        values.push(group as f64);

        let instance = DenseInstance::new(Arc::clone(&self.header), values, 1.0);

        self.produced += 1;
        Some(Box::new(instance))
    }

    fn restart(&mut self) -> Result<(), Error> {
        self.rng = StdRng::seed_from_u64(self.seed);
        self.next_class_should_be_zero = false;
        self.produced = 0;
        Ok(())
    }
}

fn build_agrawal_header() -> InstanceHeader {
    let mut attrs: Vec<AttributeRef> = Vec::new();

    attrs.push(Arc::new(NumericAttribute::new("salary".into())) as AttributeRef);
    attrs.push(Arc::new(NumericAttribute::new("commission".into())) as AttributeRef);
    attrs.push(Arc::new(NumericAttribute::new("age".into())) as AttributeRef);

    let elevel_vals: Vec<String> = (0..=4).map(|i| format!("L{i}")).collect();
    let mut elevel_map = HashMap::new();
    for (i, lab) in elevel_vals.iter().enumerate() {
        elevel_map.insert(lab.clone(), i);
    }
    attrs.push(Arc::new(NominalAttribute::with_values(
        "elevel".into(),
        elevel_vals,
        elevel_map,
    )) as AttributeRef);

    let car_vals: Vec<String> = (1..=20).map(|i| format!("C{i}")).collect();
    let mut car_map = HashMap::new();
    for (i, lab) in car_vals.iter().enumerate() {
        car_map.insert(lab.clone(), i);
    }
    attrs.push(Arc::new(NominalAttribute::with_values(
        "car".into(),
        car_vals,
        car_map,
    )) as AttributeRef);

    let z_vals: Vec<String> = (0..=8).map(|i| format!("Z{i}")).collect();
    let mut z_map = HashMap::new();
    for (i, lab) in z_vals.iter().enumerate() {
        z_map.insert(lab.clone(), i);
    }
    attrs.push(Arc::new(NominalAttribute::with_values(
        "zipcode".into(),
        z_vals,
        z_map,
    )) as AttributeRef);

    attrs.push(Arc::new(NumericAttribute::new("hvalue".into())) as AttributeRef);
    attrs.push(Arc::new(NumericAttribute::new("hyears".into())) as AttributeRef);
    attrs.push(Arc::new(NumericAttribute::new("loan".into())) as AttributeRef);

    let class_vals = vec!["groupA".into(), "groupB".into()];
    let mut class_map = HashMap::new();
    class_map.insert("groupA".into(), 0);
    class_map.insert("groupB".into(), 1);
    attrs.push(Arc::new(NominalAttribute::with_values(
        "class".into(),
        class_vals,
        class_map,
    )) as AttributeRef);

    InstanceHeader::new("agrawal".into(), attrs, 9)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attributes::NominalAttribute;
    use crate::streams::stream::Stream;

    #[test]
    fn new_rejects_invalid_function_id() {
        let err = AgrawalGenerator::new_with_id(0, false, 0.0, None, 1).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        let err = AgrawalGenerator::new_with_id(11, false, 0.0, None, 1).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn new_with_id_rejects_invalid_perturb_fraction() {
        let err = AgrawalGenerator::new_with_id(1, false, -0.01, None, 1).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        let err = AgrawalGenerator::new_with_id(1, false, 1.01, None, 1).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn header_shape_and_nominals() {
        let g = AgrawalGenerator::new_with_id(1, false, 0.0, Some(1), 42).unwrap();
        let h = g.header();
        assert_eq!(h.number_of_attributes(), 10);
        assert_eq!(h.class_index(), 9);
        assert_eq!(h.relation_name(), "agrawal");

        assert_eq!(h.attribute_at_index(0).unwrap().name(), "salary");
        assert_eq!(h.attribute_at_index(1).unwrap().name(), "commission");
        assert_eq!(h.attribute_at_index(2).unwrap().name(), "age");
        assert_eq!(h.attribute_at_index(3).unwrap().name(), "elevel");
        assert_eq!(h.attribute_at_index(4).unwrap().name(), "car");
        assert_eq!(h.attribute_at_index(5).unwrap().name(), "zipcode");
        assert_eq!(h.attribute_at_index(6).unwrap().name(), "hvalue");
        assert_eq!(h.attribute_at_index(7).unwrap().name(), "hyears");
        assert_eq!(h.attribute_at_index(8).unwrap().name(), "loan");
        assert_eq!(h.attribute_at_index(9).unwrap().name(), "class");

        let elevel = h
            .attribute_at_index(3)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(elevel.values, vec!["L0", "L1", "L2", "L3", "L4"]);
        assert_eq!(elevel.label_to_index.get("L3").copied(), Some(3));

        let car = h
            .attribute_at_index(4)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(car.values.len(), 20);
        assert_eq!(car.label_to_index.get("C1").copied(), Some(0));
        assert_eq!(car.label_to_index.get("C20").copied(), Some(19));

        let zip = h
            .attribute_at_index(5)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(
            zip.values,
            vec!["Z0", "Z1", "Z2", "Z3", "Z4", "Z5", "Z6", "Z7", "Z8"]
        );
        assert_eq!(zip.label_to_index.get("Z0").copied(), Some(0));
    }

    #[test]
    fn max_instances_and_has_more_instances() {
        let mut g = AgrawalGenerator::new_with_id(7, false, 0.0, Some(3), 123).unwrap();
        assert!(g.has_more_instances());
        assert!(g.next_instance().is_some());
        assert!(g.next_instance().is_some());
        assert!(g.next_instance().is_some());
        assert!(!g.has_more_instances());
        assert!(g.next_instance().is_none());
    }

    #[test]
    fn balance_alternates_starting_with_one_like_moa() {
        let mut g = AgrawalGenerator::new_with_id(1, true, 0.0, Some(10), 7).unwrap();
        let mut classes = Vec::new();
        for _ in 0..10 {
            let inst = g.next_instance().unwrap();
            classes.push(inst.class_value().unwrap() as u8);
        }
        for i in 0..classes.len() {
            assert_eq!(
                classes[i],
                if i % 2 == 0 { 1 } else { 0 },
                "posição {i}, seq={:?}",
                classes
            );
        }
    }

    #[test]
    fn restart_reproducible_sequence() {
        let mut g = AgrawalGenerator::new_with_id(9, false, 0.0, Some(20), 2024).unwrap();
        let a1 = g.next_instance().unwrap().to_vec();
        let a2 = g.next_instance().unwrap().to_vec();
        g.restart().unwrap();
        let b1 = g.next_instance().unwrap().to_vec();
        let b2 = g.next_instance().unwrap().to_vec();
        assert_eq!(a1, b1);
        assert_eq!(a2, b2);
    }

    #[test]
    fn perturbation_changes_numerics_when_p_is_one() {
        let mut g0 = AgrawalGenerator::new_with_id(10, false, 0.0, Some(1), 77).unwrap();
        let mut g1 = AgrawalGenerator::new_with_id(10, false, 1.0, Some(1), 77).unwrap();

        let v0 = g0.next_instance().unwrap().to_vec();
        let v1 = g1.next_instance().unwrap().to_vec();

        let differs = (v0[0] != v1[0])
            || (v0[1] != v1[1])
            || (v0[6] != v1[6])
            || (v0[8] != v1[8])
            || (v0[2] != v1[2])
            || (v0[7] != v1[7]);
        assert!(
            differs,
            "perturbation should alter some numeric attribute; v0={v0:?} v1={v1:?}"
        );
    }

    #[test]
    fn sampler_hits_both_commission_branches_with_fixed_seed() {
        let mut g = AgrawalGenerator::new_with_id(6, false, 0.0, Some(300), 424242).unwrap();
        let mut saw_zero = false;
        let mut saw_nonzero = false;
        for _ in 0..300 {
            let inst = g.next_instance().unwrap();
            let v = inst.to_vec();
            let commission = v[1];
            if commission == 0.0 {
                saw_zero = true;
            }
            if commission > 0.0 {
                saw_nonzero = true;
            }
            if saw_zero && saw_nonzero {
                break;
            }
        }
        assert!(
            saw_zero && saw_nonzero,
            "expected commission==0.0 and >0.0 in 300 samples"
        );
    }

    #[test]
    fn agrawal_function_from_id_ok_and_err() {
        for id in 1u8..=10u8 {
            let f = AgrawalFunction::try_from(id).unwrap();
            let v = f as u8;
            assert_eq!(v, id);
        }
        let err = AgrawalFunction::try_from(0).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        let err = AgrawalFunction::try_from(11).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }
}
