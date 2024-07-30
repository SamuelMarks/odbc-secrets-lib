use std::io::stdout;

use odbc_api::buffers::TextRowSet;
use odbc_api::{ConnectionOptions, Cursor, Environment, ResultSetMetadata};

use crate::error::OdbcSecretsLibError;

/// Maximum number of rows fetched with one row set. Fetching batches of rows is usually much
/// faster than fetching individual rows.
const BATCH_SIZE: usize = 5000;

pub fn odbc_runner(
    connection_string: Option<String>,
    data_source_name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    params: Option<String>,
    query: String,
) -> Result<(), OdbcSecretsLibError> {
    // Write csv to standard out
    let out = stdout();
    let mut writer = csv::Writer::from_writer(out);

    let environment = Environment::new()?;
    let connection = if connection_string.is_none() {
        environment.connect(
            data_source_name.unwrap().as_str(),
            username.unwrap().as_str(),
            password.unwrap().as_str(),
            ConnectionOptions::default(),
        )
    } else {
        environment.connect_with_connection_string(
            connection_string.unwrap().as_str(),
            ConnectionOptions::default(),
        )
    }?;
    let params = match params {
        None => (),
        Some(_params) => serde_json::from_str(_params.as_str())?,
    };

    // most of the following from https://docs.rs/odbc-api/8.1.2/odbc_api/guide/index.html
    match connection.execute(query.as_str(), params)? {
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

    Ok(())
}
