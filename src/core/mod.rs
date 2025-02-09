use std::{convert::TryInto, fs::File, path::Path};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;

use hevc_parser::hevc::{SeiMessage, USER_DATA_REGISTERED_ITU_T_35};
use hevc_parser::io::IoFormat;

pub mod parser;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("File doesn't contain dynamic metadata")]
    NoMetadataFound,
    #[error("Dynamic HDR10+ metadata detected.")]
    MetadataDetected,
}

pub fn initialize_progress_bar(format: &IoFormat, input: &Path) -> Result<ProgressBar> {
    let pb: ProgressBar;
    let bytes_count;

    if let IoFormat::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata()?;
        bytes_count = file_meta.len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.cyan} {percent}%")?,
        );
    }

    Ok(pb)
}

pub fn is_st2094_40_sei(sei_payload: &[u8], validate: bool) -> Result<bool> {
    if sei_payload.len() >= 4 {
        let sei = SeiMessage::from_bytes(sei_payload)?;

        if sei.payload_type == USER_DATA_REGISTERED_ITU_T_35 {
            // FIXME: Not sure why 4 bytes..
            let itu_t35_bytes = &sei_payload[4..];

            if itu_t35_bytes.len() >= 7 {
                let itu_t_t35_country_code = itu_t35_bytes[0];
                let itu_t_t35_terminal_provider_code =
                    u16::from_be_bytes(itu_t35_bytes[1..3].try_into()?);
                let itu_t_t35_terminal_provider_oriented_code =
                    u16::from_be_bytes(itu_t35_bytes[3..5].try_into()?);

                if itu_t_t35_country_code == 0xB5
                    && itu_t_t35_terminal_provider_code == 0x003C
                    && itu_t_t35_terminal_provider_oriented_code == 0x0001
                {
                    let application_identifier = itu_t35_bytes[5];
                    let application_version = itu_t35_bytes[6];

                    let valid_version = if validate {
                        application_version == 1
                    } else {
                        application_version <= 1
                    };

                    if application_identifier == 4 && valid_version {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}
