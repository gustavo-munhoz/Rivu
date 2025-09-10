#[inline]
pub fn current_rss_gb() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        calculate_rss_for_linux()
    }

    #[cfg(target_os = "macos")]
    {
        calculate_rss_for_macos()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn calculate_rss_for_linux() -> Option<f64> {
    use std::fs;
    let status = fs::read_to_string("/proc/self/status").ok()?;
    parse_linux_status_vm_rss_gb(&status)
}

#[cfg(target_os = "linux")]
fn parse_linux_status_vm_rss_gb(status: &str) -> Option<f64> {
    for line in status.lines() {
        let Some(rest) = line.strip_prefix("VmRSS:") else {
            continue;
        };
        if let Some(kb) = rest.split_whitespace().find_map(|t| t.parse::<u64>().ok()) {
            return Some(kb as f64 / (1024.0 * 1024.0)); // kB -> GB
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn calculate_rss_for_macos() -> Option<f64> {
    use libc::{c_int, c_void, kern_return_t, mach_msg_type_number_t, mach_port_t, time_value_t};
    use std::mem::{size_of, zeroed};

    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct mach_task_basic_info {
        virtual_size: u64,
        resident_size: u64,
        resident_size_max: u64,
        user_time: time_value_t,
        system_time: time_value_t,
        policy: i32,
        suspend_count: i32,
    }

    unsafe extern "C" {
        fn mach_task_self() -> mach_port_t;
        fn task_info(
            target_task: mach_port_t,
            flavor: c_int,
            task_info_out: *mut c_void,
            task_info_out_count: *mut mach_msg_type_number_t,
        ) -> kern_return_t;
    }

    const MACH_TASK_BASIC_INFO: c_int = 20;
    const MACH_TASK_BASIC_INFO_COUNT: mach_msg_type_number_t =
        (size_of::<mach_task_basic_info>() / size_of::<u32>()) as _;

    unsafe {
        let mut info: mach_task_basic_info = zeroed();
        let mut count = MACH_TASK_BASIC_INFO_COUNT;
        let kr = task_info(
            mach_task_self(),
            MACH_TASK_BASIC_INFO,
            &mut info as *mut _ as *mut c_void,
            &mut count,
        );
        if kr == 0 {
            return Some(info.resident_size as f64 / (1024.0 * 1024.0 * 1024.0));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    mod linux {
        use super::super::parse_linux_status_vm_rss_gb;

        #[test]
        fn parses_basic_vmrss_line() {
            let s = "Name:\tproc\nVmSize:\t  999 kB\nVmRSS:\t  123456 kB\nThreads: 4\n";
            let got = parse_linux_status_vm_rss_gb(s).unwrap();
            let want = 123456.0 / (1024.0 * 1024.0);
            assert!((got - want).abs() < 1e-12, "got={got}, want={want}");
        }

        #[test]
        fn ignores_non_numeric_tokens_and_picks_number() {
            let s = "VmRSS:\t  abc  789  kB";
            let got = parse_linux_status_vm_rss_gb(s).unwrap();
            let want = 789.0 / (1024.0 * 1024.0);
            assert!((got - want).abs() < 1e-12);
        }

        #[test]
        fn returns_none_if_missing_vmrss() {
            let s = "Name:\tfoo\nVmSize:\t 1024 kB\n";
            assert!(parse_linux_status_vm_rss_gb(s).is_none());
        }

        #[test]
        fn returns_none_if_number_missing() {
            let s = "VmRSS:\t kB";
            assert!(parse_linux_status_vm_rss_gb(s).is_none());
        }

        #[test]
        fn smoke_current_rss_non_negative() {
            let v = super::super::current_rss_gb();
            assert!(v.is_some());
            assert!(v.unwrap() >= 0.0);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_current_rss_smoke() {
        let v = current_rss_gb();
        assert!(v.is_some(), "expected Some on macOS");
        let x = v.unwrap();
        assert!(x.is_finite() && x >= 0.0, "invalid RSS value: {x}");
    }
}
