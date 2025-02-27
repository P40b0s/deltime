use std::time::Duration;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use scheduler::RepeatingStrategy;
use utilites::Date;


pub fn progress_bar_for_interval(mpb: &MultiProgress, repeating: &RepeatingStrategy, len: u32) -> ProgressBar
{
    let pb = mpb.add(ProgressBar::new(len as u64));
    pb.enable_steady_tick(Duration::from_millis(120));
    let sty = if repeating == &RepeatingStrategy::Once
    {
        ProgressStyle::with_template(
            "[{elapsed_precise}] {prefix} {spinner:.blue} {bar:40.green/cyan} {pos:>0}/{len:>0} {msg}",
        )
        .unwrap()
        .tick_strings(&[
            "▹▹▹▹▹",
            "▸▹▹▹▹",
            "▹▸▹▹▹",
            "▹▹▸▹▹",
            "▹▹▹▸▹",
            "▹▹▹▹▸",
            "▪▪▪▪▪",
            ])
        .progress_chars("●●∙")
    }
    else
    {
        ProgressStyle::with_template(
            "[{elapsed_precise}] {prefix} {spinner:.red}   {bar:40.green/cyan} {pos:>0}/{len:>0} {msg}",
        )
        .unwrap()
        .tick_strings(&[
            "∙∙∙",
			"●∙∙",
			"∙●∙",
			"∙∙●",
            "∙●∙",
			"●∙∙",
			"●●●"
            ])
        .progress_chars("●●∙")
    };
    pb.with_style(sty)
    
}
#[allow(dead_code)]
pub fn set_date_message(pb: &ProgressBar, visible: bool, date: &Date, path: &str)
{
    let d = date.format(utilites::DateFormat::DotDate);
    let t = date.format(utilites::DateFormat::Time);
    let msg= if visible
    {
        [&d, " ", &t, " -> ", path].concat()
    }
    else
    {
        [&d, " ", &t].concat()
    };
    pb.set_message(msg);
    pb.set_prefix("⌛");
}
#[allow(dead_code)]
pub fn set_interval_message(pb: &ProgressBar, visible: bool, path: &str)
{
    let msg= if visible
    {
        [" -> ", path].concat()
    }
    else
    {
        "".to_owned()
    };
    pb.set_message(msg);
    pb.set_prefix("⌛");
}


pub fn progress_bar_for_datetime(mpb: &MultiProgress, len: u32) -> ProgressBar
{

    let pb = mpb.add(ProgressBar::new(len as u64));
    pb.enable_steady_tick(Duration::from_millis(120));
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {prefix} {spinner:.blue}    {bar:40.green/cyan} [{msg}]",
    )
    .unwrap()
    .tick_strings(&[
            "🕛",
			"🕐",
			"🕑",
			"🕒",
			"🕓",
			"🕔",
			"🕕",
			"🕖",
			"🕗",
			"🕘",
			"🕙",
			"🕚",
    ])
    .progress_chars("●●∙");
    pb.with_style(sty)
}
