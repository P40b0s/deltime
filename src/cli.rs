use clap::{arg, command, Parser};

#[derive(Parser, Clone)]
#[command(version("1"), about("Аргументы"), long_about = None)]
pub struct Cli
{
  #[arg(long = "file")]
  #[arg(short = 'f')]
  #[arg(help="Полный путь к файлу")]
  pub file: String,
  #[arg(long = "interval")]
  #[arg(short = 'i')]
  #[arg(help="Интервал удаления в минутах")]
  pub interval: Option<u32>,
  #[arg(short = 'r')]
  #[arg(help="Повтор задачи, только если задан интервал")]
  pub repeat: bool,
  #[arg(long = "date")]
  #[arg(short = 'd')]
  #[arg(help="Время удаления в формате 2022-10-26T13:23:52")]
  pub date: Option<String>,
}

impl Cli
{
    pub fn parse_args() -> Option<Self>
    {
        let parsed = Self::try_parse();
        if let Ok(cli) = parsed
        {
            Some(cli)
        }
        else
        {
            //println!("Аргу: {}", parsed.err().unwrap());
            None
        }
        
    }
}
