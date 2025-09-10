#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgeBand {
    U40,
    B40_59,
    P60,
}

#[inline]
fn band_age(age: i32) -> AgeBand {
    if age < 40 {
        AgeBand::U40
    } else if age < 60 {
        AgeBand::B40_59
    } else {
        AgeBand::P60
    }
}

#[inline]
fn in_range(x: f64, lo: f64, hi: f64) -> bool {
    x >= lo && x <= hi
}
#[inline]
fn income_total(salary: f64, commission: f64) -> f64 {
    salary + commission
}
#[inline]
fn disposable_basic(salary: f64, commission: f64) -> f64 {
    2.0 * income_total(salary, commission) / 3.0
}
#[inline]
fn equity(hvalue: f64, hyears: i32) -> f64 {
    if hyears >= 20 {
        hvalue * ((hyears as f64) - 20.0) / 10.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RawAttrs {
    pub salary: f64,
    pub commission: f64,
    pub age: i32,
    pub elevel: i32,
    pub car: i32,
    pub zipcode: i32,
    pub hvalue: f64,
    pub hyears: i32,
    pub loan: f64,
}

type ClassFn = fn(&RawAttrs) -> u8;

const fn z(b: bool) -> u8 {
    if b { 0 } else { 1 }
}

/// 1: class = 0 if age < 40 or age >= 60; otherwise 1
fn rule1(a: &RawAttrs) -> u8 {
    z(matches!(band_age(a.age), AgeBand::U40 | AgeBand::P60))
}

/// 2: by age band, salary band
fn rule2(a: &RawAttrs) -> u8 {
    match band_age(a.age) {
        AgeBand::U40 => z(in_range(a.salary, 50_000.0, 100_000.0)),
        AgeBand::B40_59 => z(in_range(a.salary, 75_000.0, 125_000.0)),
        AgeBand::P60 => z(in_range(a.salary, 25_000.0, 75_000.0)),
    }
}

/// 3: by age band, elevel sets
fn rule3(a: &RawAttrs) -> u8 {
    match band_age(a.age) {
        AgeBand::U40 => z(matches!(a.elevel, 0 | 1)),
        AgeBand::B40_59 => z(matches!(a.elevel, 1 | 2 | 3)),
        AgeBand::P60 => z(matches!(a.elevel, 2 | 3 | 4)),
    }
}

/// 4: combines elevel and salary by age band
fn rule4(a: &RawAttrs) -> u8 {
    match band_age(a.age) {
        AgeBand::U40 => {
            if matches!(a.elevel, 0 | 1) {
                z(in_range(a.salary, 25_000.0, 75_000.0))
            } else {
                z(in_range(a.salary, 50_000.0, 100_000.0))
            }
        }
        AgeBand::B40_59 => {
            if matches!(a.elevel, 1 | 2 | 3) {
                z(in_range(a.salary, 50_000.0, 100_000.0))
            } else {
                z(in_range(a.salary, 75_000.0, 125_000.0))
            }
        }
        AgeBand::P60 => {
            if matches!(a.elevel, 2 | 3 | 4) {
                z(in_range(a.salary, 50_000.0, 100_000.0))
            } else {
                z(in_range(a.salary, 25_000.0, 75_000.0))
            }
        }
    }
}

/// 5: age * salary * loan
fn rule5(a: &RawAttrs) -> u8 {
    match band_age(a.age) {
        AgeBand::U40 => {
            if in_range(a.salary, 50_000.0, 100_000.0) {
                z(in_range(a.loan, 100_000.0, 300_000.0))
            } else {
                z(in_range(a.loan, 200_000.0, 400_000.0))
            }
        }
        AgeBand::B40_59 => {
            if in_range(a.salary, 75_000.0, 125_000.0) {
                z(in_range(a.loan, 200_000.0, 400_000.0))
            } else {
                z(in_range(a.loan, 300_000.0, 500_000.0))
            }
        }
        AgeBand::P60 => {
            if in_range(a.salary, 25_000.0, 75_000.0) {
                z(in_range(a.loan, 300_000.0, 500_000.0))
            } else {
                z(in_range(a.loan, 100_000.0, 300_000.0))
            }
        }
    }
}

/// 6: age * (salary+commission)
fn rule6(a: &RawAttrs) -> u8 {
    let total = income_total(a.salary, a.commission);
    match band_age(a.age) {
        AgeBand::U40 => z(in_range(total, 50_000.0, 100_000.0)),
        AgeBand::B40_59 => z(in_range(total, 75_000.0, 125_000.0)),
        AgeBand::P60 => z(in_range(total, 25_000.0, 75_000.0)),
    }
}

/// 7: 2/3 income - loan/5 - 20k
fn rule7(a: &RawAttrs) -> u8 {
    z(disposable_basic(a.salary, a.commission) - (a.loan / 5.0) - 20_000.0 > 0.0)
}

/// 8: 2/3 income - 5000*elevel - 20k
fn rule8(a: &RawAttrs) -> u8 {
    z(disposable_basic(a.salary, a.commission) - 5_000.0 * (a.elevel as f64) - 20_000.0 > 0.0)
}

/// 9: 2/3 income - 5000*elevel - loan/5 - 10k
fn rule9(a: &RawAttrs) -> u8 {
    z(disposable_basic(a.salary, a.commission)
        - 5_000.0 * (a.elevel as f64)
        - (a.loan / 5.0)
        - 10_000.0
        > 0.0)
}

/// 10: 2/3 income - 5000*elevel + equity/5 - 10k
fn rule10(a: &RawAttrs) -> u8 {
    z(
        disposable_basic(a.salary, a.commission) - 5_000.0 * (a.elevel as f64)
            + (equity(a.hvalue, a.hyears) / 5.0)
            - 10_000.0
            > 0.0,
    )
}

pub(super) static CLASS_RULES: [ClassFn; 10] = [
    rule1, rule2, rule3, rule4, rule5, rule6, rule7, rule8, rule9, rule10,
];

pub(super) fn determine(function_id: u8, a: &RawAttrs) -> u8 {
    if !(1..=10).contains(&function_id) {
        panic!("invalid function_id: {}", function_id);
    }
    CLASS_RULES[(function_id - 1) as usize](a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero() -> RawAttrs {
        RawAttrs {
            salary: 0.0,
            commission: 0.0,
            age: 0,
            elevel: 0,
            car: 0,
            zipcode: 0,
            hvalue: 0.0,
            hyears: 0,
            loan: 0.0,
        }
    }

    #[test]
    fn band_age_partitions() {
        assert!(matches!(band_age(0), AgeBand::U40));
        assert!(matches!(band_age(39), AgeBand::U40));
        assert!(matches!(band_age(40), AgeBand::B40_59));
        assert!(matches!(band_age(59), AgeBand::B40_59));
        assert!(matches!(band_age(60), AgeBand::P60));
        assert!(matches!(band_age(120), AgeBand::P60));
    }

    #[test]
    fn in_range_is_inclusive() {
        assert!(in_range(5.0, 5.0, 10.0));
        assert!(in_range(10.0, 5.0, 10.0));
        assert!(!in_range(4.999, 5.0, 10.0));
        assert!(!in_range(10.001, 5.0, 10.0));
    }

    #[test]
    fn income_and_disposable_and_equity_helpers() {
        assert_eq!(income_total(10.0, 5.0), 15.0);
        assert!((disposable_basic(12.0, 6.0) - 12.0).abs() < 1e-12); // 2/3 * (12+6) = 12
        assert_eq!(equity(300_000.0, 10), 0.0);
        assert!((equity(300_000.0, 30) - 300_000.0).abs() < 1e-9); // (30-20)/10 * hvalue
    }

    #[test]
    fn rule1_edges() {
        let mut a = zero();
        a.age = 39;
        assert_eq!(rule1(&a), 0);
        a.age = 40;
        assert_eq!(rule1(&a), 1);
        a.age = 59;
        assert_eq!(rule1(&a), 1);
        a.age = 60;
        assert_eq!(rule1(&a), 0);
    }

    #[test]
    fn rule2_salary_bands_by_age() {
        let mut a = zero();

        a.age = 20;
        a.salary = 49_999.99;
        assert_eq!(rule2(&a), 1);
        a.age = 20;
        a.salary = 50_000.0;
        assert_eq!(rule2(&a), 0);
        a.age = 20;
        a.salary = 100_000.0;
        assert_eq!(rule2(&a), 0);
        a.age = 20;
        a.salary = 100_000.01;
        assert_eq!(rule2(&a), 1);

        a.age = 50;
        a.salary = 74_999.99;
        assert_eq!(rule2(&a), 1);
        a.age = 50;
        a.salary = 75_000.0;
        assert_eq!(rule2(&a), 0);
        a.age = 50;
        a.salary = 125_000.0;
        assert_eq!(rule2(&a), 0);
        a.age = 50;
        a.salary = 125_000.01;
        assert_eq!(rule2(&a), 1);

        a.age = 70;
        a.salary = 24_999.99;
        assert_eq!(rule2(&a), 1);
        a.age = 70;
        a.salary = 25_000.0;
        assert_eq!(rule2(&a), 0);
        a.age = 70;
        a.salary = 75_000.0;
        assert_eq!(rule2(&a), 0);
        a.age = 70;
        a.salary = 75_000.01;
        assert_eq!(rule2(&a), 1);
    }

    #[test]
    fn rule3_elevel_sets_by_age() {
        let mut a = zero();

        a.age = 30;
        for e in 0..=4 {
            a.elevel = e;
            assert_eq!(rule3(&a), if e <= 1 { 0 } else { 1 });
        }

        a.age = 50;
        for e in 0..=4 {
            a.elevel = e;
            assert_eq!(rule3(&a), if (1..=3).contains(&e) { 0 } else { 1 });
        }

        a.age = 65;
        for e in 0..=4 {
            a.elevel = e;
            assert_eq!(rule3(&a), if (2..=4).contains(&e) { 0 } else { 1 });
        }
    }

    #[test]
    fn rule4_combines_elevel_and_salary() {
        let mut a = zero();

        a.age = 25;
        a.elevel = 0;
        a.salary = 50_000.0;
        assert_eq!(rule4(&a), 0);
        a.age = 25;
        a.elevel = 0;
        a.salary = 80_000.0;
        assert_eq!(rule4(&a), 1);
        a.age = 25;
        a.elevel = 2;
        a.salary = 60_000.0;
        assert_eq!(rule4(&a), 0);
        a.age = 25;
        a.elevel = 2;
        a.salary = 120_000.0;
        assert_eq!(rule4(&a), 1);

        a.age = 50;
        a.elevel = 2;
        a.salary = 80_000.0;
        assert_eq!(rule4(&a), 0);
        a.age = 50;
        a.elevel = 4;
        a.salary = 80_000.0;
        assert_eq!(rule4(&a), 0);
        a.age = 50;
        a.elevel = 4;
        a.salary = 120_000.0;
        assert_eq!(rule4(&a), 0);

        a.age = 70;
        a.elevel = 3;
        a.salary = 80_000.0;
        assert_eq!(rule4(&a), 0);
        a.age = 70;
        a.elevel = 1;
        a.salary = 80_000.0;
        assert_eq!(rule4(&a), 1);
        a.age = 70;
        a.elevel = 1;
        a.salary = 50_000.0;
        assert_eq!(rule4(&a), 0);
    }

    #[test]
    fn rule5_age_salary_loan_matrix() {
        let mut a = zero();

        a.age = 30;
        a.salary = 60_000.0;
        a.loan = 200_000.0;
        assert_eq!(rule5(&a), 0);
        a.age = 30;
        a.salary = 120_000.0;
        a.loan = 250_000.0;
        assert_eq!(rule5(&a), 0);
        a.age = 30;
        a.salary = 60_000.0;
        a.loan = 90_000.0;
        assert_eq!(rule5(&a), 1);

        a.age = 50;
        a.salary = 100_000.0;
        a.loan = 250_000.0;
        assert_eq!(rule5(&a), 0);
        a.age = 50;
        a.salary = 60_000.0;
        a.loan = 350_000.0;
        assert_eq!(rule5(&a), 0);

        a.age = 70;
        a.salary = 50_000.0;
        a.loan = 400_000.0;
        assert_eq!(rule5(&a), 0);
        a.age = 70;
        a.salary = 120_000.0;
        a.loan = 150_000.0;
        assert_eq!(rule5(&a), 0);
    }

    #[test]
    fn rule6_uses_total_income() {
        let mut a = zero();

        a.age = 25;
        a.salary = 30_000.0;
        a.commission = 30_000.0;
        assert_eq!(rule6(&a), 0);
        a.age = 25;
        a.salary = 10_000.0;
        a.commission = 10_000.0;
        assert_eq!(rule6(&a), 1);

        a.age = 50;
        a.salary = 50_000.0;
        a.commission = 25_000.0;
        assert_eq!(rule6(&a), 0);
        a.age = 70;
        a.salary = 40_000.0;
        a.commission = 30_000.0;
        assert_eq!(rule6(&a), 0);
        a.age = 70;
        a.salary = 40_000.0;
        a.commission = 35_000.0;
        assert_eq!(rule6(&a), 0);
    }

    #[test]
    fn rules_7_8_9_10_disposable_variants() {
        let mut a = zero();

        a.salary = 90_000.0;
        a.commission = 0.0;
        a.loan = 0.0;
        assert_eq!(rule7(&a), 0);
        a.loan = 500_000.0;
        assert_eq!(rule7(&a), 1);

        a.salary = 90_000.0;
        a.elevel = 0;
        assert_eq!(rule8(&a), 0);
        a.elevel = 4;
        assert_eq!(rule8(&a), 0);

        a.salary = 90_000.0;
        a.commission = 0.0;
        a.elevel = 2;
        a.loan = 0.0;
        assert_eq!(rule9(&a), 0);
        a.loan = 400_000.0;
        assert_eq!(rule9(&a), 1);

        a.salary = 0.0;
        a.commission = 0.0;
        a.elevel = 0;
        a.hvalue = 300_000.0;
        a.hyears = 10;
        assert_eq!(rule10(&a), 1);
        a.hyears = 30;
        assert_eq!(rule10(&a), 0);
    }

    #[test]
    fn determine_dispatch_table_and_bounds() {
        let a = RawAttrs {
            salary: 80_000.0,
            commission: 10_000.0,
            age: 45,
            elevel: 2,
            car: 10,
            zipcode: 3,
            hvalue: 200_000.0,
            hyears: 15,
            loan: 100_000.0,
        };
        for id in 1u8..=10u8 {
            let got = determine(id, &a);
            let expect = CLASS_RULES[(id - 1) as usize](&a);
            assert_eq!(got, expect, "id={id}");
        }
    }

    #[test]
    #[should_panic]
    fn determine_panics_on_invalid_id() {
        let a = zero();
        let _ = determine(0, &a);
    }
}
