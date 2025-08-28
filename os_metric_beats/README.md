# OS Metric Beats

Windows와 Linux 시스템에 특화된 지표 수집기 시스템입니다.

## 주요 기능

### Windows 모드
- WMI를 통한 프로세스 메모리 사용량 수집
- Windows 전용 시스템 메트릭 수집
- netstat 명령어를 통한 네트워크 패킷 정보 수집

### Linux 모드  
- /proc 파일시스템을 통한 프로세스 메모리 사용량 수집
- Linux 전용 시스템 메트릭 수집
- /proc/net 파일을 통한 네트워크 정보 수집

## 설정

`configs/system_config.toml` 파일에서 OS 버전을 설정:

### Windows 사용 시
```toml
os_server_ip = "192.168.8.77"
os_ver = "windows"
```

### Linux 사용 시
```toml
os_server_ip = "192.168.8.77" 
os_ver = "linux"
```

## 빌드 및 실행

```bash
cargo build --release
cargo run
```

## 수집 메트릭

- CPU 사용률 (최대값 및 평균값)
- 디스크 사용률
- 메모리 사용률
- 네트워크 사용량
- 프로세스 개수
- 네트워크 패킷 정보 (드롭/에러)
- 네트워크 소켓 정보 (TCP/UDP 상태)
- Java/Elasticsearch 프로세스 메모리 사용량