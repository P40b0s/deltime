use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use utilites::Date;


pub fn progress_bar_for_interval(mpb: &MultiProgress, repeating: bool, visible: bool, fp: &str, len: u32) -> ProgressBar
{
    let pb = mpb.add(ProgressBar::new(len as u64));
    let msg= if visible
    {
        [" -> ", fp].concat()
    }
    else
    {
        "".to_owned()
    };
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(msg);
    let sty = if !repeating 
    {
        ProgressStyle::with_template(
            "{prefix}[{elapsed_precise}] {spinner:.blue} {bar:40.green/cyan} {pos:>0}/{len:>0} {msg}",
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
            "{prefix}[{elapsed_precise}] {spinner:.red}   {bar:40.green/cyan} {pos:>0}/{len:>0} {msg}",
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


pub fn progress_bar_for_datetime(mpb: &MultiProgress, visible: bool, fp: &str, target_date: &Date, len: u32) -> ProgressBar
{
    let date = target_date.format(utilites::DateFormat::DotDate);
    let time = target_date.format(utilites::DateFormat::Time);
    let msg= if visible
    {
        [&date, " ", &time, " -> ", fp].concat()
    }
    else
    {
        [&date, " ", &time].concat()
    };
    let pb = mpb.add(ProgressBar::new(len as u64));
    pb.set_message(msg);
    pb.enable_steady_tick(Duration::from_millis(120));
    let sty = ProgressStyle::with_template(
        "{prefix}[{elapsed_precise}] {spinner:.blue}    {bar:40.green/cyan} [{msg}]",
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
