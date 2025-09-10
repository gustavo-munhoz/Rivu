use crate::evaluation::Snapshot;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;

pub enum CurveFormat {
    Csv,
    Tsv,
    Json,
}
pub struct LearningCurve {
    entries: Vec<Snapshot>,
}

impl LearningCurve {
    pub fn push(&mut self, snapshot: Snapshot) {
        self.entries.push(snapshot)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn latest(&self) -> Option<Snapshot> {
        self.entries.last().cloned()
    }

    pub fn export<P: AsRef<Path>>(&self, path: P, fmt: CurveFormat) -> Result<(), Error> {
        match fmt {
            CurveFormat::Csv => self.export_with_delimiter(path, ','),
            CurveFormat::Tsv => self.export_with_delimiter(path, '\t'),
            CurveFormat::Json => self.export_json(path),
        }
    }

    fn export_with_delimiter<P: AsRef<Path>>(&self, path: P, delimiter: char) -> Result<(), Error> {
        let mut w = File::create(path)?;
        writeln!(
            w,
            "instances_seen{d}accuracy{d}kappa{d}ram_hours{d}seconds",
            d = delimiter
        )?;
        for s in &self.entries {
            writeln!(
                w,
                "{}{d}{:.12}{d}{:.12}{d}{:.12}{d}{:.6}",
                s.instances_seen,
                s.accuracy,
                s.kappa,
                s.ram_hours,
                s.seconds,
                d = delimiter
            )?;
        }
        Ok(())
    }

    fn export_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut w = File::create(path)?;
        writeln!(w, "[")?;
        for (i, s) in self.entries.iter().enumerate() {
            writeln!(
                w,
                "  {{\"instances_seen\":{},\"accuracy\":{},\"kappa\":{},\"ram_hours\":{},\"seconds\":{}}}{}",
                s.instances_seen,
                s.accuracy,
                s.kappa,
                s.ram_hours,
                s.seconds,
                if i + 1 == self.entries.len() { "" } else { "," }
            )?;
        }
        writeln!(w, "]")?;
        Ok(())
    }
}

impl Default for LearningCurve {
    fn default() -> Self {
        Self { entries: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    fn snap(seen: u64, acc: f64, kap: f64, ram: f64, secs: f64) -> Snapshot {
        Snapshot {
            instances_seen: seen,
            accuracy: acc,
            kappa: kap,
            ram_hours: ram,
            seconds: secs,
        }
    }

    #[test]
    fn default_is_empty_and_latest_none() {
        let lc = LearningCurve::default();
        assert_eq!(lc.len(), 0);
        assert!(lc.latest().is_none());
    }

    #[test]
    fn push_increases_len_and_latest_returns_clone() {
        let mut lc = LearningCurve::default();
        lc.push(snap(10, 1.0, 0.5, 0.125, 2.5));
        assert_eq!(lc.len(), 1);
        let last = lc.latest().unwrap();
        assert_eq!(last.instances_seen, 10);
        assert_eq!(last.accuracy, 1.0);
        assert_eq!(last.kappa, 0.5);
        assert_eq!(last.ram_hours, 0.125);
        assert_eq!(last.seconds, 2.5);

        lc.push(snap(20, 0.25, 0.0, 1.5, 3.0));
        assert_eq!(lc.len(), 2);
        let last = lc.latest().unwrap();
        assert_eq!(last.instances_seen, 20);
        assert_eq!(last.accuracy, 0.25);
        assert_eq!(last.kappa, 0.0);
        assert_eq!(last.ram_hours, 1.5);
        assert_eq!(last.seconds, 3.0);
    }

    #[test]
    fn export_csv_with_two_rows() {
        let mut lc = LearningCurve::default();
        lc.push(snap(10, 1.0, 0.5, 0.125, 2.5));
        lc.push(snap(20, 0.25, 0.0, 1.5, 3.0));

        let tf = NamedTempFile::new().unwrap();
        lc.export(tf.path(), CurveFormat::Csv).unwrap();

        let got = fs::read_to_string(tf.path()).unwrap();
        let exp = "\
instances_seen,accuracy,kappa,ram_hours,seconds
10,1.000000000000,0.500000000000,0.125000000000,2.500000
20,0.250000000000,0.000000000000,1.500000000000,3.000000
";
        assert_eq!(got, exp);
    }

    #[test]
    fn export_tsv_with_two_rows() {
        let mut lc = LearningCurve::default();
        lc.push(snap(10, 1.0, 0.5, 0.125, 2.5));
        lc.push(snap(20, 0.25, 0.0, 1.5, 3.0));

        let tf = NamedTempFile::new().unwrap();
        lc.export(tf.path(), CurveFormat::Tsv).unwrap();

        let got = fs::read_to_string(tf.path()).unwrap();
        let exp = "\
instances_seen\taccuracy\tkappa\tram_hours\tseconds
10\t1.000000000000\t0.500000000000\t0.125000000000\t2.500000
20\t0.250000000000\t0.000000000000\t1.500000000000\t3.000000
";
        assert_eq!(got, exp);
    }

    #[test]
    fn export_json_with_two_rows() {
        let mut lc = LearningCurve::default();
        lc.push(snap(10, 1.0, 0.5, 0.125, 2.5));
        lc.push(snap(20, 0.25, 0.0, 1.5, 3.0));

        let tf = NamedTempFile::new().unwrap();
        lc.export(tf.path(), CurveFormat::Json).unwrap();

        let got = fs::read_to_string(tf.path()).unwrap();
        let exp = "\
[
  {\"instances_seen\":10,\"accuracy\":1,\"kappa\":0.5,\"ram_hours\":0.125,\"seconds\":2.5},
  {\"instances_seen\":20,\"accuracy\":0.25,\"kappa\":0,\"ram_hours\":1.5,\"seconds\":3}
]
";
        assert_eq!(got, exp);
    }

    #[test]
    fn export_empty_csv_and_json() {
        let lc = LearningCurve::default();

        let tf_csv = NamedTempFile::new().unwrap();
        lc.export(tf_csv.path(), CurveFormat::Csv).unwrap();
        let got_csv = fs::read_to_string(tf_csv.path()).unwrap();
        let exp_csv = "instances_seen,accuracy,kappa,ram_hours,seconds\n";
        assert_eq!(got_csv, exp_csv);

        let tf_tsv = NamedTempFile::new().unwrap();
        lc.export(tf_tsv.path(), CurveFormat::Tsv).unwrap();
        let got_tsv = fs::read_to_string(tf_tsv.path()).unwrap();
        let exp_tsv = "instances_seen\taccuracy\tkappa\tram_hours\tseconds\n";
        assert_eq!(got_tsv, exp_tsv);

        let tf_json = NamedTempFile::new().unwrap();
        lc.export(tf_json.path(), CurveFormat::Json).unwrap();
        let got_json = fs::read_to_string(tf_json.path()).unwrap();
        let exp_json = "[\n]\n";
        assert_eq!(got_json, exp_json);
    }
}
