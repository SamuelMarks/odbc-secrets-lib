use std::io::stdout;

use odbc_api::buffers::TextRowSet;
use odbc_api::{ConnectionOptions, Cursor, Environment, ResultSetMetadata};

#[cfg(feature = "clap")]
use clap::ValueEnum;

use crate::error::OdbcSecretsLibError;

/// Maximum number of rows fetched with one row set. Fetching batches of rows is usually much
/// faster than fetching individual rows.
const BATCH_SIZE: usize = 5000;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum OutputFormat {
    #[default]
    CSV,
    JSON,
    PARQUET,
}

#[cfg(feature = "clap")]
impl OutputFormat {
    /// Report all `possible_values`
    pub fn possible_values() -> impl Iterator<Item = clap::builder::PossibleValue> {
        Self::value_variants()
            .iter()
            .filter_map(clap::ValueEnum::to_possible_value)
    }
}

#[cfg(feature = "clap")]
impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[cfg(feature = "clap")]
impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::CSV, Self::JSON, Self::PARQUET]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::CSV => clap::builder::PossibleValue::new("csv"),
            Self::JSON => clap::builder::PossibleValue::new("json"),
            Self::PARQUET => clap::builder::PossibleValue::new("parquet"),
        })
    }
}

pub fn odbc_runner(
    connection_string: Option<String>,
    params: Option<String>,
    query: String,
    output_format: OutputFormat,
) -> Result<(), OdbcSecretsLibError> {
    // Write csv to standard out
    let environment = Environment::new()?;
    let connection = environment.connect_with_connection_string(
            connection_string.unwrap().as_str(),
            ConnectionOptions::default(),
    );
    let params = match params {
        None => (),
        Some(_params) => serde_json::from_str(_params.as_str())?,
    };

    let out = stdout();
    match output_format {
        OutputFormat::CSV => {
            let mut writer = csv::Writer::from_writer(out);

            // most of the following from https://docs.rs/odbc-api/8.1.2/odbc_api/guide/index.html
            match connection?.execute(query.as_str(), params)? {
                Some(mut cursor) => {
                    // Write the column names to stdout
                    let headline: Vec<String> = cursor.column_names()?.collect::<Result<_, _>>()?;
                    writer.write_record(headline)?;

                    // Use schema in cursor to initialize a text buffer large enough to hold the largest
                    // possible strings for each column up to an upper limit of 4KiB.
                    let mut buffers = TextRowSet::for_cursor(BATCH_SIZE, &mut cursor, Some(4096))?;
                    // Bind the buffer to the cursor. It is now being filled with every call to fetch.
                    let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

                    // Iterate over batches
                    while let Some(batch) = row_set_cursor.fetch()? {
                        // Within a batch, iterate over every row
                        for row_index in 0..batch.num_rows() {
                            // Within a row iterate over every column
                            let record = (0..batch.num_cols())
                                .map(|col_index| batch.at(col_index, row_index).unwrap_or(&[]));
                            // Writes row as csv
                            writer.write_record(record)?;
                        }
                    }
                }
                None => {
                    eprintln!("Query came back empty. No output has been created.");
                }
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}
