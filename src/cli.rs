use crate::dates::Timeframe;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "shittd")]
#[command(about="A manager for your shit to do list", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    /// Adds one or more tasks to the list
    Add {
        #[arg(value_name = "TASK_NAME")]
        tasks: Vec<String>,
        /// Optionally provide a calendar date for this task
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Lists incomplete and completed tasks
    #[command(aliases = ["ls"])]
    List {
        #[arg(value_enum, default_value = "today")]
        timeframe: Timeframe,
    },
    #[command(arg_required_else_help = true)]
    /// Finishes one or many tasks by ID
    Finish {
        #[arg(value_name = "TASK_ID")]
        task_id: Vec<u8>,
    },
    /// Pushes task(s) off to the following day, or optionally a specific calendar date
    #[command(arg_required_else_help = true)]
    Push {
        #[arg(value_name = "TASK_ID")]
        tasks: Vec<u8>,

        #[arg(short, long)]
        date: Option<String>,
    },
    /// Removes completed tasks from the list
    Clean,
}
