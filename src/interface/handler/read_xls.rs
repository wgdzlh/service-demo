use std::io;

use axum::{
    extract::{Multipart, State},
    Json,
};
use futures::TryStreamExt;
use tokio::{
    fs::{self, File},
    io::BufWriter,
};
use tokio_util::io::StreamReader;

use crate::{
    app::{log::*, utils},
    infrastructure::shell::ChildWorker,
    interface::{dto::MultipartFile, resp::*},
    repository::{Error, Result},
};

use super::to_raw_resp;

/// Parse xls File
///
/// Try to parse a xls File.
#[utoipa::path(
        post,
        path = "/xls/parse",
        request_body(content = inline(MultipartFile), description = "xls file to parse", content_type = "multipart/form-data"),
        responses(
            (status = 200, description = "Xls File parsed successfully", body = VoidRes)
        )
    )]
pub async fn parse(
    parser: State<ChildWorker>,
    mut multipart: Multipart,
) -> Result<Json<ObjectRes>> {
    let filename = utils::get_uuid_str() + ".xls";
    let mut created_file = false;
    let mut file_size = 0;
    while let Some(field) = multipart.next_field().await? {
        if matches!(field.name(), Some("file")) {
            // let data = field.bytes().await?;
            // if !data.is_empty() {
            //     filename = utils::get_uuid_str() + ".xls";
            //     fs::write(&filename, data).await?;
            // }
            // stream to file, save lots of memory if file is big
            let body_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
            let body_reader = StreamReader::new(body_with_io_error);
            futures::pin_mut!(body_reader);

            let mut file = BufWriter::new(File::create(&filename).await?);
            created_file = true;
            file_size = tokio::io::copy(&mut body_reader, &mut file).await?;
            break;
        }
    }
    if file_size == 0 {
        if created_file {
            fs::remove_file(filename).await.ok();
        }
        return Err(Error::BadRequest);
    }
    info!(?filename, "processing xls file");
    let ret = parser
        .submit(format!(r#"{{"file": "{filename}"}}"#))
        .await?;
    debug!(len=%ret.len(), "parse ret detail");
    fs::remove_file(filename).await.ok();
    Ok(Json(to_raw_resp(ret)?))
}
