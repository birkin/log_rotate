#[macro_use]
extern crate log;

extern crate chrono;  // <https://docs.rs/crate/chrono/0.4.11>
extern crate env_logger;
extern crate glob;  // <https://docs.rs/glob/0.3.0/glob/>

use chrono::{DateTime, Local};
use env_logger::{Builder, Target};
use glob::glob;

use serde::Deserialize;
use serde_json::{Value};

use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};


/*

NEXT:
- next:
    - separate code into own repo.

- someday:
    - pass in size-check as arg
    - consider more capable logger
        ... or consider updating a hash or an array that writes to a log-file

*/


#[derive(Deserialize, Debug)]
struct Config {
    log_level: String,
    logger_json_file_path: String,
    max_entries: i8  // this could be added to the json-file instead
}

impl Config {
    /*  forgive the "RUST_LOG" hack; i really wanted to use the envar project-prefix to set the log-level,
        ...and couldn't figure out how to specify an alternative prefix for env_logger::init() */
    fn new() -> Config {
        match envy::prefixed("LOG_ROTATOR__").from_env::<Config>() {  // https://github.com/softprops/envy
            Ok(config) => {
                env::set_var( "RUST_LOG", &config.log_level);
                let log_level = config.log_level;  // not used, but still useful to set, for panic-message if it's missing
                let logger_json_file_path = config.logger_json_file_path;
                let max_entries = config.max_entries;
                Config { log_level, logger_json_file_path, max_entries }
            },
            Err(error) => panic!("{:#?}", error) // this shows the missing envar
        }
    }
}


fn main() {

    /* start */
    let start_time = Instant::now();
    let local_time: DateTime<Local> = Local::now();
    // println!( "\nstarting rust-custom-logrotate code at, ``{:?}``", local_time.to_rfc3339() );

    /* setup settings */
    let config = Config::new();
    // println!("config, ``{:?}``", config);

    /* setup logging */
    let mut log_builder = Builder::from_default_env();
    log_builder.target( Target::Stdout );
    log_builder.init();
    info!( "{}", format!("\n\nstarting rust-custom-logrotate code at, ``{:?}``", local_time.to_rfc3339()) );
    debug!( "{}", format!("config, ``{:#?}``", config) );  // debug! needs a string literal  :(

    /* load log-paths json-object */
    let log_paths_obj: std::vec::Vec<serde_json::value::Value> = load_log_paths( &config.logger_json_file_path );
    debug!( "{}", format!("log_paths_obj, ``{:?}``", log_paths_obj) );

    /* process files */
    process_logs( &log_paths_obj );

    /* output */
    let duration: Duration = start_time.elapsed();
    info!( "{}", format!("elapsed-time, ``{:?}``", duration) );
    // println!( "elapsed-time, ``{:?}``", duration );

}


fn process_logs( log_paths_obj: &std::vec::Vec<serde_json::value::Value> ) {
    /*  Iterates through the log_paths_obj, sending each item to a function...
        ...which will manage the steps of processing the item.
        Called by: main() */
    for item in log_paths_obj {
        // println!("\nitem, ``{:?}``", item);  // yields (EG): item, ``Object({"path": String("/foo/the.log")})``
        // let z: () = item;  // yields: found `&serde_json::value::Value`
        manage_directory_entry( item );
    }
}


fn manage_directory_entry( item: &serde_json::value::Value ) {
    /*  Manages the steps to process the log entry.
        Called by: process_logs()
        Steps...
        - check whether file exists and bail if it doesn't.
        - check file size and bail if it's not big enough.
        - determine file-name and parent-directory from path.
            ...they're needed to determine the list of relevant log-files to process.
            ...this is because some log-directories contain more than one set of log files.
            ...example, a 'usep' log-directory may contain a set of usep-webapp logs, and a set of usep-processing logs.
        - read all the files in the directory.
        - create loop to process each relevant log-file.
        */

    // debug!( "{}", format!("item from within manage_directory_entry, ``{:?}``", item) );  // yields (EG): item, ``Object({"path": String("/foo/the.log")})``

    // -- get path String from json-dict-object
    let path_value = &item["path"];
    let path_rfrnc = path_value.as_str().unwrap_or_else(|| {
        panic!("problem reading path from json-obj: {:?}", path_value);
    });
    // let zz: () = path_rfrnc;  // yields: found `&str`
    let path: String = path_rfrnc.into();
    // let zz: () = path;  // yields: found struct `std::string::String`

    // -- does file exist?
    if check_existence( &path ) == false {
        return;
    }

    // -- is it big enough to process?
    if check_big_enough( &path ) == false {
        return;
    }

    info!( "{}", format!("PROCEEDING to process path, ``{:?}``", path) );

    // -- get file_name
    let file_name: String = make_file_name( &path );

    // -- get parent-path
    //  TODO...
    //  Try something like: let mut parent_path = std::Path;
    //  Then maybe sending the empty parent_path to the prep-function and returning it won't cause lifetime errors.
    //  ...but getting a String works for now; so this try will be a refactor.
    let parent_path: String = determine_directory( &path );

    // -- get list of relevant log-file-paths
    let file_list: Vec<String> = prep_file_list( &parent_path, &file_name );
    println!("file_list, ``{:?}``", file_list);

    // -- process each log-file
    for file in file_list {
        process_file( &file, &file_name, &parent_path )
    }

}  // end fn manage_directory_entry()


fn process_file( file_path: &str, file_name: &str, parent_path: &str ) {
    /*  Examines given file and deletes it or backs it up.
        Called by: manage_directory_entry()
        Steps...
        - determine the extension.
        - delete oldest file if necessary.
        - determine new extension.
        - create destination filepath and copy source filepath to it.
        - create the new empty file if necessary.
        */

    debug!( "{}", format!("processing file_path, ``{:?}``", file_path) );

    // -- determine the extension
    let path = Path::new( file_path );
    let extension = path.extension().unwrap_or_else( || {
        panic!("could not determine extension");
    });
    println!("extension, ``{:?}``", extension);
    // let zz: () = extension;  // yields: found `&std::ffi::OsStr`
    let extension_str = extension.to_str().unwrap_or_else( || {
        panic!("could not convert to &str");
    });
    println!("extension_str, ``{:?}``", extension_str);
    // let zz: () = extension_str;  // yields; found `&str`

    // -- delete the oldest file if necessary
    if extension_str == "9" {
        debug!( "{}", format!("about to try deleting file") );
        fs::remove_file( file_path ).unwrap_or_else( |err| {
            panic!("could not delete old file; error, ``{}``", err);
        });
        info!( "{}", format!("file successfully deleted") );
        return;
    }

    // -- determine new extension
    let new_extension: String = match extension_str {
        "log" => "0".to_string(),
        "0" => "1".to_string(),
        "1" => "2".to_string(),
        "2" => "3".to_string(),
        "3" => "4".to_string(),
        "4" => "5".to_string(),
        "5" => "6".to_string(),
        "6" => "7".to_string(),
        "7" => "8".to_string(),
        "8" => "9".to_string(),
        _ => {
            let err_message = "unexpected extension found".to_string();
            error!( "{}", err_message );
            // panic!( err_message );
            panic!( "{}", err_message );
        }
    };
    debug!( "{}", format!("new_extension, ``{:?}``", new_extension) );
    // let zz: () = new_extension;  // yields: found struct `std::string::String`

    // -- now that we have the new extension, create the destination path
    let destination_path = format!( "{}/{}.{}", parent_path, file_name, new_extension );
    debug!( "{}", format!("destination_path, ``{:?}``", destination_path) );

    // -- and now copy
    let bytes_copied = fs::copy( file_path, destination_path ).unwrap_or_else( |err| {
        let err_message = format!( "problem copying the file, ``{}``", err );
        error!( "{}", err_message );
        // panic!( err_message );
        panic!( "{}", err_message );
    });
    info!( "{}", format!("copied ``{:?}K``", (bytes_copied / 1024)) );

    // -- finally, create the new empty file if necessary
    if extension_str == "log" {
        info!( "{}", format!("creating new empty file") );
        let _empty_file = File::create(&path).unwrap_or_else( |err| {
            let err_message = format!( "problem creating new empty file, ``{}``", err );
            error!( "{}", err_message );
            // panic!( err_message );
            panic!( "{}", err_message );
        });
    }

} // end fn process_file()


fn prep_file_list( parent_path: &str, file_name: &str ) -> Vec<String> {
    /*  Examines the directory for the target file-path and returns a list of all the log-entries.
        Called by: manage_directory_entry() */

    // -- initialize the holder
    let mut v: std::vec::Vec<String> = Vec::new();

    // -- create the glob pattern
    let pattern = format!( "{}/*{}*", parent_path, file_name );
    debug!( "{}", format!("pattern, ``{:?}``", pattern) );

    // -- apply the pattern
    let paths = glob( &pattern ).unwrap_or_else( |err| {
        panic!("could not glob the pattern; error, ``{}``", err);
    });
    // let zz: () = paths;  // yields (before unwrap): found enum `std::result::Result<glob::Paths, glob::PatternError>`

    // -- convert each glob::Path into a String and add it to the Vector holder
    for entry in paths {
        let path = entry.unwrap_or_else( |err| {  // path without unwrap is: enum `std::result::Result<std::path::PathBuf, glob::GlobError>`
            panic!("could not access the path; error, ``{}``", err);
        });
        // let zz: () = path;  // yields: found struct `std::path::PathBuf`

        let path_str = path.to_str().unwrap_or_else( || {
            panic!("could turn the path into a string");
        });
        // let zz: () = path_str;  // yields: found `&str`

        let path_string: String = path_str.into();
        debug!( "{}", format!("path_string, ``{:?}``", path_string) );
        // let zz: () = path_string;  // yields: found struct `std::string::String`

        v.push( path_string );
    }

    info!( "{}", format!("log-files before sort, ``{:#?}``", v) );
    // let zz: () = v; // yields: found struct `std::vec::Vec<std::string::String>`

    v.sort();  // may not need this initial sort
    v.reverse();
    info!( "{}", format!("log-files after sort, ``{:#?}``", v) );

    v

} // end fn fn prep_file_list()


fn determine_directory(  path: &str ) -> String {
    /*  Takes full-path string-reference & determines the path to the parent-directory.
        Called by: manage_directory_entry() */

    // -- path-ify the string-reference and get the parent path-obj
    let parent = Path::new(path).parent().unwrap_or_else( || {
        panic!("no parent found");
    });
    // let zz: () = parent;  // yields: found `&std::path::Path`

    // -- turn the path-obj into a string-reference on the way to getting a String
    let parent_str = parent.to_str().unwrap_or_else( || {
        panic!("could not get &str from parent-Path");
    });
    // let zz: () = parent_str;  // yields: found `&str`

    // -- get the String
    let parent_string: String = parent_str.into();  // TODO: combine this and the Option() step above.
    // let zz: () = parent_string;  // yields: found struct `std::string::String`  👍
    debug!( "{}", format!("parent_string, ``{:?}``", parent_string) );

    parent_string
}


fn make_file_name( path: &str) -> String {
    /*  Extracts filename from path
        Called by: manage_directory_entry() */

    let file_name_osstr = Path::new(path).file_name().unwrap_or_else( || {
        panic!("could not determine filename");
    });
    // let zz: () = file_name_osstr;  // yields: found `&std::ffi::OsStr`

    let file_name_str = file_name_osstr.to_str().unwrap_or_else( || {
        panic!("could not derive file_name_str fro file_name_osstr");
    });
    // let zz: () = file_name_str;  // yields: found `&str`

    let file_name_string: String = file_name_str.into();
    // let zz: () = file_name_string; // yields: found struct `std::string::String`

    debug!( "{}", format!("file_name_string, ``{:?}``", file_name_string) );
    file_name_string
}


fn check_big_enough( path: &str ) -> bool {
    /*  Checks that file is big enough.
        Called by: manage_directory_entry().
        TODO: check against config setting */

    const THRESHOLD: u64 = 250;
    let mut result = false;

    let metadata = fs::metadata(path);
    // println!("metadata, ``{:?}``", metadata);

    match metadata {
        Ok(metadata) => {
            let file_size: u64 = metadata.len() / 1000;
            debug!( "{}", format!("file_size in Kb, ``{}``", file_size) );
            // let zz: () = file_size;  // yields: found `u64`
            if file_size > THRESHOLD {
                debug!( "file_size big enough to process" );
                result = true;
            } else {
                debug!( "file_size not big enough to process" );
            }
        },
        Err(err) => {
            error!( "{}", format!("could not get metadata for path, ``{}``; error, ``{}``", path, err) );
        }
    };

    return result;
}


fn check_existence( path: &str ) -> bool {
    /*  Checks that file exists.
        Called by: manage_directory_entry() */
    if Path::new(path).exists() == false {
        error!( "{}", format!("path, ``{}`` does not exist", path) );
        false
    } else {
        debug!( "{}", format!("path, ``{}`` exists", path) );
        true
    }
}


fn load_log_paths( logger_json_file_path: &std::string::String ) -> std::vec::Vec<serde_json::value::Value> {
    /*  Loads json list of paths into an iterable array.
        Called by: main()  */

    // --- read file ---
    let jsn: String = fs::read_to_string( &logger_json_file_path ).unwrap_or_else(|error| {
        panic!("Problem reading the json-file -- ``{:?}``", error);
    });
    // println!("\njsn, ``{:?}``", jsn);  // yields: jsn, ``"[\n  {\n    \"path\": \"/foo/the.log\"\n  },\n  {\n    \"path\": \"/path/to/logs/addto_refworks_logs/addto_refworks.log\"\n  },\n  {\n    \"path\": \"/path/to/logs/annex_counts_logs/annex_counts.log\"\n  }\n]\n"``
    // let zz: () = jsn;  // yields: found struct `std::string::String`

    // --- turn String into json-object ---
    let paths_obj: Value = serde_json::from_str(&jsn).unwrap_or_else(|error| {
        panic!("Problem converting the json-file to an object -- maybe invalid json? -- ``{:?}``", error);
    });
    // println!("\npaths_obj, ``{:?}``", paths_obj); // yields: paths_obj, ``Array([Object({"path": String("/foo/the.log")}), Object({"path": String("/path/to/logs/addto_refworks_logs/addto_refworks.log")}), Object({"path": String("/path/to/logs/annex_counts_logs/annex_counts.log")})])``
    // let zz: () = paths_obj;  // yields: found enum `serde_json::value::Value`

    // --- turns the json-object in to a Vector(reference) ---
    // Question: why wasn't I able to iterate over this?
    let paths_obj_array = paths_obj.as_array().unwrap_or_else(|| {  // as_array() returns Option -- <https://docs.serde.rs/serde_json/value/enum.Value.html#method.as_array>
        panic!("Problem handling paths_obj");
    });
    // println!("\npaths_obj_array, ``{:?}``", paths_obj_array);  // yields: paths_obj_array, ``[Object({"path": String("/foo/the.log")}), Object({"path": String("/path/to/logs/addto_refworks_logs/addto_refworks.log")}), Object({"path": String("/path/to/logs/annex_counts_logs/annex_counts.log")})]``
    // let zz: () = paths_obj_array;  // yields found reference `&std::vec::Vec<serde_json::value::Value>`

    // -- turns the Vector-reference into a Vector-Struct
    // Only this allowed me to pass the returned-result to another function: process_logs()
    // Just skimmed a _great_ post that I should re-read to refactor this function: <https://hermanradtke.com/2015/06/22/effectively-using-iterators-in-rust.html>
    let real_array = paths_obj_array.to_vec();
    // println!("\nreal_array, ``{:?}``", real_array);  // yields: real_array, ``[Object({"path": String("/foo/the.log")}), Object({"path": String("/path/to/logs/addto_refworks_logs/addto_refworks.log")}), Object({"path": String("/path/to/logs/annex_counts_logs/annex_counts.log")})]``
    // let zz: () = real_array;  // yields: found struct `std::vec::Vec<serde_json::value::Value>`

    return real_array;
}
