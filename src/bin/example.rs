use korea_investment_api::types::config::Config;
use korea_investment_api::types::Account;
use korea_investment_api::KoreaInvestmentApi;
use std::io::Read;
use std::path::PathBuf;
use clap::Parser;
use thiserror::Error;
use env_logger;

#[derive(Parser)]
#[command(name = "opt", about = "example")]
struct Opt {
    config_path: PathBuf,
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),
    #[error(transparent)]
    ApiError(#[from] korea_investment_api::Error),
}

fn get_config(path: &PathBuf) -> Result<Config, Error> {
    let mut buf = String::new();
    let mut fd = std::fs::File::open(path)?;
    let _len = fd.read_to_string(&mut buf)?;
    Ok(toml::from_str(&buf)?)
}

async fn get_api(config: &Config) -> Result<KoreaInvestmentApi, Error> {
    let account = Account {
        cano: config.cano().clone(),
        acnt_prdt_cd: config.acnt_prdt_cd().clone(),
    };
    Ok(KoreaInvestmentApi::new(
        config.environment().clone(),
        config.app_key(),
        config.app_secret(),
        account,
        config.hts_id(),
        config.token_as_option(),
        config.approval_key_as_option(),
    )
    .await?)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let Opt { config_path } = Opt::parse();
    let config = get_config(&config_path).unwrap();
    let api = get_api(&config).await.unwrap();

    // 주식잔고조회 예시
    let balance = api
        .order
        .inquire_balance(
            "N", // afhr_flpr_yn: 시간외단일가 여부 (N: 기본값)
            "02", // inqr_dvsn: 조회구분 (02: 종목별)
            "01", // unpr_dvsn: 단가구분 (01: 기본값)
            "N", // fund_sttl_icld_yn: 펀드결제분포함여부 (N: 미포함)
            "N", // fncg_amt_auto_rdpt_yn: 융자금액자동상환여부 (N: 기본값)
            "00", // prcs_dvsn: 처리구분 (00: 전일매매포함)
            None,  // ctx_area_fk100: 연속조회검색조건100 (None: 최초조회)
            None,  // ctx_area_nk100: 연속조회키100 (None: 최초조회)
        )
        .await
        .unwrap();
    println!("주식잔고조회 결과: {:?}", balance);

    // 매수가능조회 예시
    let buying_power = api
        .order
        .inquire_psbl_order(
            "005930", // pdno: 종목코드 (삼성전자)
            "",  // ord_unpr: 주문단가 (시장가 시 "0" 또는 "" 입력)
            "01",     // ord_dvsn: 주문구분 (01: 시장가, 00: 지정가)
            "N",      // cma_evlu_amt_icld_yn: CMA평가금액포함여부 (Y: 포함, N: 미포함)
            "N",      // ovrs_icld_yn: 해외포함여부 (Y: 포함, N: 미포함)
        )
        .await
        .unwrap();
    println!("매수가능조회 결과: {:?}", buying_power);
}
