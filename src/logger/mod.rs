pub mod log {
    use colored::Colorize;
    use std::fmt::Debug;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::Path;

    pub enum Level {
        Debug,
        Warning,
        Error,
    }

    pub fn log<T>(level: Level, message: &str, handle: &T)
    where T: Debug + Send {
        let tag = match level {
            Level::Debug => {
                let tag = "[Debug]";
                println!("{} {} Value: {:?}", tag.blue(), message, handle);
                tag
            },
            Level::Warning => {
                let tag = "[WARN]";
                println!("{} {} Value: {:?}", tag.yellow(), message, handle);
                tag
            },
            Level::Error => {
                let tag = "[ERROR]";
                println!("{} {} Value: {:?}", tag.red(), message, handle);
                tag
            }, 
        };

        let moscow_time = (
            chrono::Utc::now() + chrono::Duration::hours(3)
        )
        .format("%d-%m-%Y")
        .to_string();
        
        if !Path::new("logs").exists() {
            std::fs::create_dir("logs").unwrap();
        }
        
        let path = format!("logs/{}.txt", moscow_time);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .unwrap();

        let moscow_time = (
            chrono::Utc::now() + chrono::Duration::hours(3)
        )
        .format("%d-%m-%Y %H:%M:%S");

        writeln!(&mut file, "{} | {} {}\nSource value: {:#?}", moscow_time, tag, message, handle).unwrap();
    }
}