use crate::common::*;

#[doc = "Json 파일을 읽어서 객체로 변환해주는 함수 - deprecated"]
/// # Arguments
/// * `file_path` - 읽을대상 파일이 존재하는 경로
///
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_json_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    let file: File = File::open(file_path)?;
    let reader: BufReader<File> = BufReader::new(file);
    let data: T = from_reader(reader)?;

    Ok(data)
}

#[doc = "Serialize 가능한 데이터를 json 파일로 저장해주는 함수"]
pub fn save_as_json<T: Serialize>(data: &T, path: &str) -> Result<(), anyhow::Error> {
    let save_file: File = File::create(path)?;
    serde_json::to_writer_pretty(save_file, data)?;
    Ok(())
}

#[doc = "특정 파일을 로드하고 특정 데이터로 해당 파일을 덮어씌워주는 함수"]
pub fn load_or_create_file<I, T, F>(
    input: &I,
    file_path: &str,
    create_fn: F,
) -> Result<T, anyhow::Error>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce(&I) -> T,
{
    match read_json_from_file::<T>(file_path) {
        Ok(state) => Ok(state),
        Err(e) => {
            warn!("[WARN] Failed to load state: {:?}, creating new state", e);
            let new_state: T = create_fn(input);
            save_as_json::<T>(&new_state, file_path)?;
            Ok(new_state)
        }
    }
}

#[doc = "toml 파일을 읽어서 객체로 변환해주는 함수"]
/// # Arguments
/// * `file_path` - 읽을 대상 toml 파일이 존재하는 경로
///
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_toml_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    let toml_content: String = std::fs::read_to_string(file_path)?;
    let toml: T = toml::from_str(&toml_content)?;

    Ok(toml)
}

#[doc = "특정 파일의 경로를 받으면 그 파일을 열어서 contents 를 읽어주는 함수 -> 해당 파일이 숫자인 경우에만 작동하도록 함"]
/// # Arguments
/// * `path` - 읽을 대상 toml 파일이 존재하는 경로
///
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_u64<P: AsRef<std::path::Path>>(path: P) -> Result<u64, anyhow::Error> {
    Ok(fs::read_to_string(path)?.trim().parse().unwrap_or(0))
}
