use anyhow::Result;
use comfy_table::{ContentArrangement, Table};
use linapi::modules::{LoadedModule, ModuleFile};
use std::path::{Path, PathBuf};
use structopt::{clap::AppSettings, StructOpt};

#[derive(Debug, StructOpt)]
enum Commands {
    /// List loaded kernel modules
    List {},

    /// Load a kernel module
    Insert {
        /// Path to file, or module name.
        ///
        /// If this is a full path, that file is loaded directly.
        ///
        /// If a file name, `lib/modules/(uname -r)` will be searched.
        module: PathBuf,

        /// Force load the module.
        ///
        /// This ignores important kernel compatibility checks.
        ///
        /// USE AT YOUR OWN RISK
        #[structopt(long, short)]
        force: bool,
    },

    /// Remove/unload a kernel module
    Remove {
        /// Name of the module to unload
        name: String,

        /// Force unload the module.
        ///
        /// This is extremely dangerous and can cause serious system
        /// instability. Not all modules are designed to be unloaded, and this
        /// can cause a module currently being used to be unloaded.
        ///
        /// USE AT YOUR OWN RISK
        #[structopt(long, short)]
        force: bool,
    },

    /// Get information on a kernel module
    Info {
        /// Name of the module, or a path to one.
        module: PathBuf,
    },
}

/// Manage Linux Kernel Modules
#[derive(Debug, StructOpt)]
#[structopt(global_settings(&[
        AppSettings::ColoredHelp,
        AppSettings::GlobalVersion,
        AppSettings::DisableVersion,
    ]
))]
struct Args {
    #[structopt(subcommand)]
    cmd: Commands,
}

fn get_module(name: &Path) -> ModuleFile {
    if name.is_absolute() {
        ModuleFile::from_path(name)
    } else {
        ModuleFile::from_name(name.to_str().unwrap())
    }
}

fn list_modules() {
    let mut table = Table::new();
    if let None = table.get_table_width() {
        table.set_table_width(80);
    }
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Module", "Size (Bytes)", "References", "Used By"]);

    for m in LoadedModule::from_loaded() {
        table.add_row(vec![
            m.name(),
            &m.size().to_string(),
            &m.ref_count().unwrap().to_string(),
            &m.holders()
                .iter()
                .map(|m| m.name())
                .collect::<Vec<_>>()
                .join("\n"),
        ]);
    }

    println!("{}", table);
}

fn add_module(name: &Path, force: bool) {
    let m = get_module(name);
    if force {
        unsafe { m.force_load("") };
    } else {
        m.load("");
    }
}

fn remove_module(name: &str, force: bool) {
    let m = LoadedModule::from_name(name);
    if force {
        unsafe { m.force_unload() };
    } else {
        m.unload();
    }
}

fn info_module(name: &Path) {
    let mut table = Table::new();
    if let None = table.get_table_width() {
        table.set_table_width(80);
    }
    table.set_content_arrangement(ContentArrangement::Dynamic);
    //
    let m = get_module(name);
    let info = m.info();
    //
    table.set_header(vec![
        //
        "File".into(),
        m.path().display().to_string(),
    ]);

    table.add_row(vec!["Authors", &info.authors.join("\n")]);
    table.add_row(vec!["License", &info.license]);
    table.add_row(vec!["Description", &info.description]);
    table.add_row(vec!["Version", &info.version]);
    table.add_row(vec!["Firmware", &info.firmware.join("\n")]);
    table.add_row(vec!["Alias", &info.alias.join("\n")]);
    table.add_row(vec!["Dependencies", &info.dependencies.join("\n")]);
    table.add_row(vec![
        "Soft Dependencies".into(),
        info.soft_dependencies.join("\n"),
    ]);
    table.add_row(vec!["In Tree", &info.in_tree.to_string()]);
    table.add_row(vec!["Retpoline", &info.retpoline.to_string()]);
    table.add_row(vec!["Staging", &info.staging.to_string()]);
    table.add_row(vec!["Version Magic", &info.version_magic]);
    table.add_row(vec!["Source Checksum", &info.source_checksum]);
    //
    let mut p_table = Table::new();
    p_table.set_header(vec!["Name", "Desc", "Type"]);
    p_table.set_table_width(
        table.get_table_width().unwrap()
            - table.column_iter().next().unwrap().get_max_content_width()
            // 6 Is how many characters, including padding, the first column borders take.
            // Plus 1 for our own padding, for a total of 7.
            - 7,
    );
    p_table.set_content_arrangement(ContentArrangement::Dynamic);
    for p in info.parameters {
        p_table.add_row(vec![p.name, p.description, p.type_]);
    }
    table.add_row(vec!["Parameters", &p_table.to_string()]);
    //
    // let mut s_table = Table::new();
    // s_table.set_header(vec!["Signer", "ID", "Key", "Hash Algorithm",
    // "Signature"]); s_table.set_table_width(
    //     table.get_table_width().unwrap()
    //         - table.column_iter().next().unwrap().get_max_content_width()
    //         // 6 Is how many characters, including padding, the first column
    // borders take.         // Plus 1 for our own padding, for a total of 7.
    //         - 7,
    // );
    // s_table.set_content_arrangement(ContentArrangement::Dynamic);
    // let _s = dbg!(m.signature().unwrap());
    // s_table.add_row(vec!["Test"]);

    table.add_row(vec!["Signature", &m.has_signature().to_string()]);

    //
    println!("{}", table);
}

fn main() -> Result<()> {
    let args = Args::from_args();
    //
    match args.cmd {
        Commands::List { .. } => list_modules(),
        Commands::Insert { module, force } => add_module(&module, force),
        Commands::Remove { name, force } => remove_module(&name, force),
        Commands::Info { module } => info_module(&module),
    }
    //
    Ok(())
}