#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process,
};

use clap::{Parser, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};

const DEFAULT_OUTPUT: &str = ".env";
const DEFAULT_SITE: &str = "localhost";
const DEFAULT_REGION: &str = "eu-west-1";
const DEFAULT_BUCKET: &str = "hdrop";
const DEFAULT_SINGLE_FILE_LIMIT_MB: &str = "500";
const DEFAULT_CACHE_MEMORY_LIMIT_MB: &str = "2000";
const DEFAULT_CACHE_DISK_LIMIT_MB: &str = "20000";
const DEFAULT_CACHE_DIR: &str = "/cache/hdrop";
const DEFAULT_LOCAL_STORAGE_DIR: &str = "/data/hdrop-files";
const DEFAULT_LOCAL_STORAGE_LIMIT_MB: &str = "100000";
const DEFAULT_POSTGRES_USER: &str = "hdrop";
const DEFAULT_POSTGRES_DB: &str = "hdrop";
const DEFAULT_POSTGRES_HOST: &str = "postgres";
const DEFAULT_POSTGRES_PORT: &str = "5432";
const TODO_S3_ACCESS_KEY_ID: &str = "TODO_SET_S3_ACCESS_KEY_ID";
const TODO_S3_SECRET_ACCESS_KEY: &str = "TODO_SET_S3_SECRET_ACCESS_KEY";
const TODO_POSTGRES_PASSWORD: &str = "TODO_SET_POSTGRES_PASSWORD";
const PASSWORD_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum StorageProvider {
    S3,
    Local,
}

impl StorageProvider {
    fn as_env(self) -> &'static str {
        match self {
            Self::S3 => "s3",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum S3AddressingStyle {
    Path,
    Virtual,
}

impl S3AddressingStyle {
    fn as_env(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Virtual => "virtual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CacheStrategy {
    Memory,
    Disk,
    Hybrid,
}

impl CacheStrategy {
    fn as_env(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum DatabaseMode {
    Bundled,
    External,
}

#[derive(Debug, Parser)]
#[command(
    about = "Generate a production Docker Compose .env file for hdrop.",
    long_about = "Generate a production Docker Compose .env file for hdrop.\n\nBy default, this command opens an interactive setup flow.",
    after_help = "Examples:\n  cargo run -p hdrop-env --\n  cargo run -p hdrop-env -- --site hdrop.io\n  cargo run -p hdrop-env -- --site hdrop.io --storage-provider local\n  cargo run -p hdrop-env -- --site hdrop.io --database-mode external --postgres-host db.example.com\n  cargo run -p hdrop-env -- --site hdrop.io --cache-strategy disk --non-interactive"
)]
struct Cli {
    /// Public hostname, without a scheme. Default: localhost
    #[arg(long, value_name = "DOMAIN", default_value = DEFAULT_SITE)]
    site: Option<String>,

    /// Storage provider. Default: local
    #[arg(long, value_enum, value_name = "PROVIDER")]
    storage_provider: Option<StorageProvider>,

    /// S3 access key ID.
    #[arg(long, value_name = "VALUE")]
    s3_access_key_id: Option<String>,

    /// S3 secret access key.
    #[arg(long, value_name = "VALUE")]
    s3_secret_access_key: Option<String>,

    /// S3 region. Default: eu-west-1
    #[arg(long, value_name = "REGION")]
    s3_region: Option<String>,

    /// S3 addressing style. Virtual style requires the bucket in the endpoint
    /// hostname. Default: path
    #[arg(long, value_enum, value_name = "STYLE")]
    s3_addressing_style: Option<S3AddressingStyle>,

    /// Optional overall S3 request timeout in seconds. Unset disables it.
    #[arg(long, value_name = "SECONDS", value_parser = clap::value_parser!(u64).range(1..))]
    s3_request_timeout_secs: Option<u64>,

    /// S3 API endpoint, without the bucket path. With virtual style, include the
    /// bucket in the hostname. Legacy endpoints ending in `/<bucket>` remain accepted.
    /// Default: https://s3.<region>.amazonaws.com
    #[arg(long, value_name = "URL")]
    s3_endpoint: Option<String>,

    /// S3 bucket name. Default: hdrop
    #[arg(long, value_name = "NAME")]
    s3_bucket: Option<String>,

    /// Public URL prefix for uploaded files. Default: <s3-endpoint>/<bucket>
    #[arg(long, value_name = "URL")]
    s3_public_url: Option<String>,

    /// Local storage directory. Default: /data/hdrop-files
    #[arg(long, value_name = "PATH")]
    local_storage_dir: Option<String>,

    /// Local storage limit in MB. Default: 100000
    #[arg(long, value_name = "MB")]
    local_storage_limit_mb: Option<String>,

    /// Cache strategy. Default: disk
    #[arg(long, value_enum, value_name = "STRATEGY")]
    cache_strategy: Option<CacheStrategy>,

    /// Memory cache limit in MB. Default: 2000
    #[arg(long, value_name = "MB")]
    cache_memory_limit_mb: Option<String>,

    /// Disk cache limit in MB. Default: 20000
    #[arg(long, value_name = "MB")]
    cache_disk_limit_mb: Option<String>,

    /// Disk cache directory. Default: /cache/hdrop
    #[arg(long, value_name = "PATH")]
    cache_dir: Option<String>,

    /// Database mode. Default: bundled
    #[arg(long, value_enum, value_name = "MODE")]
    database_mode: Option<DatabaseMode>,

    /// Postgres host. Default: postgres
    #[arg(long, value_name = "HOST")]
    postgres_host: Option<String>,

    /// Postgres port. Default: 5432
    #[arg(long, value_name = "PORT")]
    postgres_port: Option<String>,

    /// Postgres user. Default: hdrop
    #[arg(long, value_name = "NAME")]
    postgres_user: Option<String>,

    /// Postgres password. Default: generated for bundled, TODO placeholder for external
    #[arg(long, value_name = "VALUE")]
    postgres_password: Option<String>,

    /// Postgres database. Default: hdrop
    #[arg(long, value_name = "NAME")]
    postgres_db: Option<String>,

    /// Upload limit in MB. Default: 500
    #[arg(long, value_name = "MB")]
    single_file_limit_mb: Option<String>,

    /// Output path. Default: .env
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Overwrite the output file if it exists.
    #[arg(long)]
    force: bool,

    /// Print to stdout instead of writing a file.
    #[arg(long)]
    stdout: bool,

    /// Use defaults and flags without prompting.
    #[arg(long)]
    non_interactive: bool,
}

#[derive(Debug)]
struct Config {
    output: PathBuf,
    site: String,
    storage_provider: StorageProvider,
    s3_access_key_id: String,
    s3_secret_access_key: String,
    s3_region: String,
    s3_addressing_style: S3AddressingStyle,
    s3_request_timeout_secs: Option<u64>,
    s3_endpoint: String,
    s3_bucket: String,
    s3_public_url: String,
    local_storage_dir: String,
    local_storage_limit_mb: String,
    cache_strategy: CacheStrategy,
    cache_memory_limit_mb: String,
    cache_disk_limit_mb: String,
    cache_dir: String,
    database_mode: DatabaseMode,
    postgres_host: String,
    postgres_port: String,
    postgres_user: String,
    postgres_password: Option<String>,
    postgres_db: String,
    single_file_limit_mb: String,
    force: bool,
    stdout: bool,
    non_interactive: bool,
}

impl Default for Config {
    fn default() -> Self {
        let s3_region = DEFAULT_REGION.to_string();
        let s3_bucket = DEFAULT_BUCKET.to_string();
        let s3_addressing_style = S3AddressingStyle::Path;
        let s3_endpoint = default_s3_endpoint(&s3_region, &s3_bucket, s3_addressing_style);
        let s3_public_url = default_s3_public_url(&s3_endpoint, &s3_bucket, s3_addressing_style);

        Self {
            output: PathBuf::from(DEFAULT_OUTPUT),
            site: DEFAULT_SITE.to_string(),
            storage_provider: StorageProvider::Local,
            s3_access_key_id: TODO_S3_ACCESS_KEY_ID.to_string(),
            s3_secret_access_key: TODO_S3_SECRET_ACCESS_KEY.to_string(),
            s3_region,
            s3_addressing_style,
            s3_request_timeout_secs: None,
            s3_endpoint,
            s3_bucket,
            s3_public_url,
            local_storage_dir: DEFAULT_LOCAL_STORAGE_DIR.to_string(),
            local_storage_limit_mb: DEFAULT_LOCAL_STORAGE_LIMIT_MB.to_string(),
            cache_strategy: CacheStrategy::Disk,
            cache_memory_limit_mb: DEFAULT_CACHE_MEMORY_LIMIT_MB.to_string(),
            cache_disk_limit_mb: DEFAULT_CACHE_DISK_LIMIT_MB.to_string(),
            cache_dir: DEFAULT_CACHE_DIR.to_string(),
            database_mode: DatabaseMode::Bundled,
            postgres_host: DEFAULT_POSTGRES_HOST.to_string(),
            postgres_port: DEFAULT_POSTGRES_PORT.to_string(),
            postgres_user: DEFAULT_POSTGRES_USER.to_string(),
            postgres_password: None,
            postgres_db: DEFAULT_POSTGRES_DB.to_string(),
            single_file_limit_mb: DEFAULT_SINGLE_FILE_LIMIT_MB.to_string(),
            force: false,
            stdout: false,
            non_interactive: false,
        }
    }
}

impl Config {
    fn from_cli(cli: Cli) -> Self {
        let mut config = Self::default();
        let changed_s3_region = cli.s3_region.is_some();
        let changed_s3_addressing_style = cli.s3_addressing_style.is_some();
        let changed_s3_endpoint = cli.s3_endpoint.is_some();
        let changed_s3_bucket = cli.s3_bucket.is_some();

        if let Some(output) = cli.output {
            config.output = output;
        }
        if let Some(site) = cli.site {
            config.site = normalize_site(site);
        }
        if let Some(storage_provider) = cli.storage_provider {
            config.storage_provider = storage_provider;
        }
        if let Some(s3_access_key_id) = cli.s3_access_key_id {
            config.s3_access_key_id = s3_access_key_id;
        }
        if let Some(s3_secret_access_key) = cli.s3_secret_access_key {
            config.s3_secret_access_key = s3_secret_access_key;
        }
        if let Some(s3_region) = cli.s3_region {
            config.s3_region = s3_region;
        }
        if let Some(s3_addressing_style) = cli.s3_addressing_style {
            config.s3_addressing_style = s3_addressing_style;
        }
        if let Some(s3_request_timeout_secs) = cli.s3_request_timeout_secs {
            config.s3_request_timeout_secs = Some(s3_request_timeout_secs);
        }
        if let Some(s3_bucket) = cli.s3_bucket {
            config.s3_bucket = s3_bucket;
        }
        if let Some(s3_endpoint) = cli.s3_endpoint {
            config.s3_endpoint = s3_endpoint;
        } else if changed_s3_region || changed_s3_addressing_style || changed_s3_bucket {
            config.s3_endpoint = default_s3_endpoint(
                &config.s3_region,
                &config.s3_bucket,
                config.s3_addressing_style,
            );
        }
        if let Some(s3_public_url) = cli.s3_public_url {
            config.s3_public_url = s3_public_url;
        } else if changed_s3_region
            || changed_s3_addressing_style
            || changed_s3_endpoint
            || changed_s3_bucket
        {
            config.s3_public_url = default_s3_public_url(
                &config.s3_endpoint,
                &config.s3_bucket,
                config.s3_addressing_style,
            );
        }
        if let Some(local_storage_dir) = cli.local_storage_dir {
            config.local_storage_dir = local_storage_dir;
        }
        if let Some(local_storage_limit_mb) = cli.local_storage_limit_mb {
            config.local_storage_limit_mb = local_storage_limit_mb;
        }
        if let Some(cache_strategy) = cli.cache_strategy {
            config.cache_strategy = cache_strategy;
        }
        if let Some(cache_memory_limit_mb) = cli.cache_memory_limit_mb {
            config.cache_memory_limit_mb = cache_memory_limit_mb;
        }
        if let Some(cache_disk_limit_mb) = cli.cache_disk_limit_mb {
            config.cache_disk_limit_mb = cache_disk_limit_mb;
        }
        if let Some(cache_dir) = cli.cache_dir {
            config.cache_dir = cache_dir;
        }
        if let Some(database_mode) = cli.database_mode {
            config.database_mode = database_mode;
        }
        if let Some(postgres_host) = cli.postgres_host {
            config.postgres_host = postgres_host;
        }
        if let Some(postgres_port) = cli.postgres_port {
            config.postgres_port = postgres_port;
        }
        if let Some(postgres_user) = cli.postgres_user {
            config.postgres_user = postgres_user;
        }
        if let Some(postgres_password) = cli.postgres_password {
            config.postgres_password = Some(postgres_password);
        }
        if let Some(postgres_db) = cli.postgres_db {
            config.postgres_db = postgres_db;
        }
        if let Some(single_file_limit_mb) = cli.single_file_limit_mb {
            config.single_file_limit_mb = single_file_limit_mb;
        }

        config.force = cli.force;
        config.stdout = cli.stdout;
        config.non_interactive = cli.non_interactive || cli.stdout;

        config
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let mut config = Config::from_cli(cli);

    if !config.non_interactive && !config.stdout {
        prompt_for_config(&mut config)?;
    }

    validate_config(&config)?;
    let content = render_env(&mut config)?;

    if config.stdout {
        print!("{content}");
        return Ok(());
    }

    if config.output.exists() && !config.force {
        return Err(format!(
            "{} already exists. Use --force to overwrite it, --stdout to print the template, or answer yes in interactive mode.",
            config.output.display()
        ));
    }

    write_output(&config.output, &content)?;
    println!("Wrote {}", config.output.display());
    Ok(())
}

fn prompt_for_config(config: &mut Config) -> Result<(), String> {
    let theme = ColorfulTheme::default();

    println!("hdrop production environment setup");
    println!("Press Enter to accept a default shown in brackets.\n");

    config.output = PathBuf::from(prompt_text(
        &theme,
        "Output file",
        &config.output.display().to_string(),
    )?);
    config.site = normalize_site(prompt_text(&theme, "Public hostname", &config.site)?);

    config.storage_provider = prompt_storage_provider(&theme, config.storage_provider)?;
    match config.storage_provider {
        StorageProvider::S3 => prompt_s3(&theme, config)?,
        StorageProvider::Local => prompt_local_storage(&theme, config)?,
    }

    config.cache_strategy = prompt_cache_strategy(&theme, config.cache_strategy)?;
    match config.cache_strategy {
        CacheStrategy::Memory => {
            config.cache_memory_limit_mb = prompt_text(
                &theme,
                "Cache memory limit in MB",
                &config.cache_memory_limit_mb,
            )?;
        }
        CacheStrategy::Disk => {
            config.cache_disk_limit_mb = prompt_text(
                &theme,
                "Cache disk limit in MB",
                &config.cache_disk_limit_mb,
            )?;
            config.cache_dir = prompt_text(&theme, "Cache directory", &config.cache_dir)?;
        }
        CacheStrategy::Hybrid => {
            config.cache_memory_limit_mb = prompt_text(
                &theme,
                "Cache memory limit in MB",
                &config.cache_memory_limit_mb,
            )?;
            config.cache_disk_limit_mb = prompt_text(
                &theme,
                "Cache disk limit in MB",
                &config.cache_disk_limit_mb,
            )?;
            config.cache_dir = prompt_text(&theme, "Cache directory", &config.cache_dir)?;
        }
    }

    config.database_mode = prompt_database_mode(&theme, config.database_mode)?;
    prompt_database(&theme, config)?;

    println!("\nRuntime limits");
    config.single_file_limit_mb = prompt_text(
        &theme,
        "Single file limit in MB",
        &config.single_file_limit_mb,
    )?;

    if config.output.exists() && !config.force {
        config.force = prompt_confirm(
            &theme,
            &format!("{} already exists. Overwrite it?", config.output.display()),
            false,
        )?;
    }

    Ok(())
}

fn prompt_database(theme: &ColorfulTheme, config: &mut Config) -> Result<(), String> {
    println!("\nPostgres");
    match config.database_mode {
        DatabaseMode::Bundled => {
            config.postgres_host = DEFAULT_POSTGRES_HOST.to_string();
            config.postgres_port = DEFAULT_POSTGRES_PORT.to_string();
            config.postgres_user = prompt_text(theme, "Postgres user", &config.postgres_user)?;
            config.postgres_db = prompt_text(theme, "Postgres database", &config.postgres_db)?;
            config.postgres_password = prompt_optional_password(
                theme,
                "Postgres password",
                "leave blank to generate a secure random password",
            )?;
        }
        DatabaseMode::External => {
            let default_host = if config.postgres_host == DEFAULT_POSTGRES_HOST {
                "db.example.com"
            } else {
                &config.postgres_host
            };
            config.postgres_host = prompt_text(theme, "Postgres host", default_host)?;
            config.postgres_port = prompt_text(theme, "Postgres port", &config.postgres_port)?;
            config.postgres_user = prompt_text(theme, "Postgres user", &config.postgres_user)?;
            config.postgres_db = prompt_text(theme, "Postgres database", &config.postgres_db)?;
            config.postgres_password = Some(prompt_secret_required(theme, "Postgres password")?);
        }
    }

    Ok(())
}

fn prompt_s3(theme: &ColorfulTheme, config: &mut Config) -> Result<(), String> {
    println!("\nS3-compatible storage");
    config.s3_access_key_id = prompt_text(theme, "S3 access key ID", &config.s3_access_key_id)?;
    config.s3_secret_access_key =
        prompt_secret_with_fallback(theme, "S3 secret access key", &config.s3_secret_access_key)?;
    config.s3_region = prompt_text(theme, "S3 region", &config.s3_region)?;
    config.s3_bucket = prompt_text(theme, "S3 bucket name", &config.s3_bucket)?;
    config.s3_addressing_style = prompt_s3_addressing_style(theme, config.s3_addressing_style)?;
    config.s3_endpoint = prompt_text(
        theme,
        "S3 endpoint",
        &default_s3_endpoint(
            &config.s3_region,
            &config.s3_bucket,
            config.s3_addressing_style,
        ),
    )?;
    let default_public_url = default_s3_public_url(
        &config.s3_endpoint,
        &config.s3_bucket,
        config.s3_addressing_style,
    );
    config.s3_public_url = prompt_text(theme, "S3 public URL", &default_public_url)?;
    Ok(())
}

fn prompt_s3_addressing_style(
    theme: &ColorfulTheme,
    current: S3AddressingStyle,
) -> Result<S3AddressingStyle, String> {
    let items = [
        "Path style - endpoint/bucket/object",
        "Virtual-hosted style - bucket.endpoint/object",
    ];
    let default = match current {
        S3AddressingStyle::Path => 0,
        S3AddressingStyle::Virtual => 1,
    };
    let selected = Select::with_theme(theme)
        .with_prompt("How should S3 buckets be addressed?")
        .items(items)
        .default(default)
        .interact()
        .map_err(|err| format!("failed to read S3 addressing style: {err}"))?;

    Ok(match selected {
        0 => S3AddressingStyle::Path,
        _ => S3AddressingStyle::Virtual,
    })
}

fn prompt_local_storage(theme: &ColorfulTheme, config: &mut Config) -> Result<(), String> {
    println!("\nLocal file storage");
    config.local_storage_dir =
        prompt_text(theme, "Local storage directory", &config.local_storage_dir)?;
    config.local_storage_limit_mb = prompt_text(
        theme,
        "Local storage limit in MB",
        &config.local_storage_limit_mb,
    )?;
    Ok(())
}

fn prompt_storage_provider(
    theme: &ColorfulTheme,
    current: StorageProvider,
) -> Result<StorageProvider, String> {
    let items = [
        "S3-compatible object store",
        "Local disk storage on this server",
    ];
    let default = match current {
        StorageProvider::S3 => 0,
        StorageProvider::Local => 1,
    };
    let selected = Select::with_theme(theme)
        .with_prompt("Where should uploaded files be stored?")
        .items(&items)
        .default(default)
        .interact()
        .map_err(|err| format!("failed to read storage provider: {err}"))?;

    Ok(match selected {
        0 => StorageProvider::S3,
        _ => StorageProvider::Local,
    })
}

fn prompt_cache_strategy(
    theme: &ColorfulTheme,
    current: CacheStrategy,
) -> Result<CacheStrategy, String> {
    let items = [
        "Memory cache - fastest, no temporary files on disk",
        "Disk cache - recommended for most servers",
        "Hybrid cache - memory first, then disk",
    ];
    let default = match current {
        CacheStrategy::Memory => 0,
        CacheStrategy::Disk => 1,
        CacheStrategy::Hybrid => 2,
    };
    let selected = Select::with_theme(theme)
        .with_prompt("How should in-flight uploads be cached?")
        .items(&items)
        .default(default)
        .interact()
        .map_err(|err| format!("failed to read cache strategy: {err}"))?;

    Ok(match selected {
        0 => CacheStrategy::Memory,
        1 => CacheStrategy::Disk,
        _ => CacheStrategy::Hybrid,
    })
}

fn prompt_database_mode(
    theme: &ColorfulTheme,
    current: DatabaseMode,
) -> Result<DatabaseMode, String> {
    let items = ["Bundled Postgres container", "External Postgres server"];
    let default = match current {
        DatabaseMode::Bundled => 0,
        DatabaseMode::External => 1,
    };
    let selected = Select::with_theme(theme)
        .with_prompt("Which Postgres database should hdrop use?")
        .items(&items)
        .default(default)
        .interact()
        .map_err(|err| format!("failed to read database mode: {err}"))?;

    Ok(match selected {
        0 => DatabaseMode::Bundled,
        _ => DatabaseMode::External,
    })
}

fn prompt_text(theme: &ColorfulTheme, label: &str, default: &str) -> Result<String, String> {
    Input::<String>::with_theme(theme)
        .with_prompt(label)
        .default(default.to_string())
        .interact_text()
        .map_err(|err| format!("failed to read {label}: {err}"))
}

fn prompt_secret_required(theme: &ColorfulTheme, label: &str) -> Result<String, String> {
    loop {
        let value = Password::with_theme(theme)
            .with_prompt(label)
            .allow_empty_password(false)
            .interact()
            .map_err(|err| format!("failed to read {label}: {err}"))?;

        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
}

fn prompt_secret_with_fallback(
    theme: &ColorfulTheme,
    label: &str,
    fallback: &str,
) -> Result<String, String> {
    let value = Password::with_theme(theme)
        .with_prompt(format!(
            "{label} (leave blank to keep current/default value)"
        ))
        .allow_empty_password(true)
        .interact()
        .map_err(|err| format!("failed to read {label}: {err}"))?;

    Ok(if value.is_empty() {
        fallback.to_string()
    } else {
        value
    })
}

fn prompt_optional_password(
    theme: &ColorfulTheme,
    label: &str,
    hint: &str,
) -> Result<Option<String>, String> {
    let value = Password::with_theme(theme)
        .with_prompt(format!("{label} ({hint})"))
        .allow_empty_password(true)
        .interact()
        .map_err(|err| format!("failed to read {label}: {err}"))?;

    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn prompt_confirm(theme: &ColorfulTheme, label: &str, default: bool) -> Result<bool, String> {
    Confirm::with_theme(theme)
        .with_prompt(label)
        .default(default)
        .interact()
        .map_err(|err| format!("failed to read confirmation: {err}"))
}

fn validate_config(config: &Config) -> Result<(), String> {
    require_non_empty("site", &config.site)?;
    require_positive_integer("single file limit", &config.single_file_limit_mb)?;
    require_non_empty("Postgres host", &config.postgres_host)?;
    require_positive_integer("Postgres port", &config.postgres_port)?;
    require_non_empty("postgres user", &config.postgres_user)?;
    require_non_empty("postgres database", &config.postgres_db)?;
    if let Some(password) = &config.postgres_password {
        require_non_empty("Postgres password", password)?;
    }

    match config.storage_provider {
        StorageProvider::S3 => {
            require_non_empty("S3 region", &config.s3_region)?;
            require_non_empty("S3 endpoint", &config.s3_endpoint)?;
            require_non_empty("S3 bucket", &config.s3_bucket)?;
            require_non_empty("S3 public URL", &config.s3_public_url)?;
        }
        StorageProvider::Local => {
            require_non_empty("local storage directory", &config.local_storage_dir)?;
            require_positive_integer("local storage limit", &config.local_storage_limit_mb)?;
        }
    }

    match config.cache_strategy {
        CacheStrategy::Memory => {
            require_positive_integer("cache memory limit", &config.cache_memory_limit_mb)?;
        }
        CacheStrategy::Disk => {
            require_positive_integer("cache disk limit", &config.cache_disk_limit_mb)?;
            require_non_empty("cache directory", &config.cache_dir)?;
        }
        CacheStrategy::Hybrid => {
            require_positive_integer("cache memory limit", &config.cache_memory_limit_mb)?;
            require_positive_integer("cache disk limit", &config.cache_disk_limit_mb)?;
            require_non_empty("cache directory", &config.cache_dir)?;
        }
    }

    Ok(())
}

fn require_non_empty(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{label} must not be empty"))
    } else {
        Ok(())
    }
}

fn require_positive_integer(label: &str, value: &str) -> Result<(), String> {
    match value.parse::<usize>() {
        Ok(number) if number > 0 => Ok(()),
        _ => Err(format!("{label} must be a positive integer")),
    }
}

fn normalize_site(site: String) -> String {
    site.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

fn default_s3_endpoint(region: &str, bucket: &str, addressing_style: S3AddressingStyle) -> String {
    match addressing_style {
        S3AddressingStyle::Path => format!("https://s3.{region}.amazonaws.com"),
        S3AddressingStyle::Virtual => format!("https://{bucket}.s3.{region}.amazonaws.com"),
    }
}

fn default_s3_public_url(
    endpoint: &str,
    bucket: &str,
    addressing_style: S3AddressingStyle,
) -> String {
    match addressing_style {
        S3AddressingStyle::Path => format!("{}/{}", endpoint.trim_end_matches('/'), bucket),
        S3AddressingStyle::Virtual => endpoint.trim_end_matches('/').to_string(),
    }
}

fn percent_encode_connection_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());

    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}

fn quote_compose_env(value: &str) -> String {
    // Compose only treats single-quoted .env values as literal strings.
    format!("'{}'", value.replace('\'', "\\'"))
}

fn render_env(config: &mut Config) -> Result<String, String> {
    let public_url = format!("https://{}", config.site);
    let postgres_password = match (&config.database_mode, &config.postgres_password) {
        (_, Some(password)) => password.clone(),
        (DatabaseMode::Bundled, None) => {
            let generated = random_hex(PASSWORD_BYTES)?;
            config.postgres_password = Some(generated.clone());
            generated
        }
        (DatabaseMode::External, None) => TODO_POSTGRES_PASSWORD.to_string(),
    };
    let database_user = percent_encode_connection_component(&config.postgres_user);
    let database_password = percent_encode_connection_component(&postgres_password);
    let database_name = percent_encode_connection_component(&config.postgres_db);
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database_user, database_password, config.postgres_host, config.postgres_port, database_name
    );

    let mut env = format!(
        "\
# Generated by hdrop-env. Review every TODO value before deploying.

# Docker Compose
",
    );

    if config.database_mode == DatabaseMode::Bundled {
        env.push_str("COMPOSE_PROFILES=bundled-postgres\n");
    }

    env.push_str(&format!(
        "\n\
# Public deployment
HDROP_SITE_ADDRESS={site}
HDROP_PUBLIC_URL={public_url}

# Caddy
HDROP_MAX_UPLOAD_SIZE={single_file_limit}MB

# Database
# If bundled Postgres has already initialized its Docker volume, changing these
# values will not update the stored database credentials automatically.
DATABASE_URL={database_url}
",
        site = quote_compose_env(&config.site),
        public_url = quote_compose_env(&public_url),
        single_file_limit = config.single_file_limit_mb,
        database_url = quote_compose_env(&database_url),
    ));

    if config.database_mode == DatabaseMode::Bundled {
        env.push_str(&format!(
            "\
POSTGRES_USER={postgres_user}
POSTGRES_PASSWORD={postgres_password}
POSTGRES_DB={postgres_db}
",
            postgres_user = quote_compose_env(&config.postgres_user),
            postgres_password = quote_compose_env(&postgres_password),
            postgres_db = quote_compose_env(&config.postgres_db),
        ));
    }

    env.push_str(&format!(
        "\n\
# Storage
STORAGE_PROVIDER={storage_provider}
",
        storage_provider = config.storage_provider.as_env(),
    ));

    match config.storage_provider {
        StorageProvider::S3 => {
            let s3_request_timeout = config
                .s3_request_timeout_secs
                .map(|seconds| format!("S3_REQUEST_TIMEOUT_SECS={seconds}\n"))
                .unwrap_or_else(|| {
                    "# Optional overall S3 request timeout. Unset disables it.\n\
# S3_REQUEST_TIMEOUT_SECS=300\n"
                        .to_string()
                });
            env.push_str(&format!(
                "\
S3_ACCESS_KEY_ID={s3_access_key_id}
S3_SECRET_ACCESS_KEY={s3_secret_access_key}
S3_REGION={s3_region}
# Use path (default) or virtual. Virtual requires the bucket in the endpoint hostname.
S3_ADDRESSING_STYLE={s3_addressing_style}
{s3_request_timeout}\
S3_ENDPOINT={s3_endpoint}
S3_BUCKET_NAME={s3_bucket}
S3_PUBLIC_URL={s3_public_url}
",
                s3_access_key_id = quote_compose_env(&config.s3_access_key_id),
                s3_secret_access_key = quote_compose_env(&config.s3_secret_access_key),
                s3_region = quote_compose_env(&config.s3_region),
                s3_addressing_style = config.s3_addressing_style.as_env(),
                s3_request_timeout = s3_request_timeout,
                s3_endpoint = quote_compose_env(&config.s3_endpoint),
                s3_bucket = quote_compose_env(&config.s3_bucket),
                s3_public_url = quote_compose_env(&config.s3_public_url),
            ));
        }
        StorageProvider::Local => {
            env.push_str(&format!(
                "\
LOCAL_STORAGE_DIR={local_storage_dir}
LOCAL_STORAGE_LIMIT_MB={local_storage_limit_mb}
",
                local_storage_dir = quote_compose_env(&config.local_storage_dir),
                local_storage_limit_mb = config.local_storage_limit_mb,
            ));
        }
    }

    env.push_str(&format!(
        "\n# Cache\nCACHE_STRATEGY={cache_strategy}\n",
        cache_strategy = config.cache_strategy.as_env(),
    ));

    match config.cache_strategy {
        CacheStrategy::Memory => {
            env.push_str(&format!(
                "CACHE_MEMORY_LIMIT_MB={}\n",
                config.cache_memory_limit_mb
            ));
        }
        CacheStrategy::Disk => {
            env.push_str(&format!(
                "\
CACHE_DISK_LIMIT_MB={cache_disk_limit}
CACHE_DIR={cache_dir}
",
                cache_disk_limit = config.cache_disk_limit_mb,
                cache_dir = quote_compose_env(&config.cache_dir),
            ));
        }
        CacheStrategy::Hybrid => {
            env.push_str(&format!(
                "\
CACHE_MEMORY_LIMIT_MB={cache_memory_limit}
CACHE_DISK_LIMIT_MB={cache_disk_limit}
CACHE_DIR={cache_dir}
",
                cache_memory_limit = config.cache_memory_limit_mb,
                cache_disk_limit = config.cache_disk_limit_mb,
                cache_dir = quote_compose_env(&config.cache_dir),
            ));
        }
    }

    env.push_str(&format!(
        "\n# Runtime limits\nSINGLE_FILE_LIMIT_MB={single_file_limit}\n",
        single_file_limit = config.single_file_limit_mb,
    ));

    Ok(env)
}

fn write_output(output: &PathBuf, content: &str) -> Result<(), String> {
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create output directory {}: {err}",
                parent.display()
            )
        })?;
    }

    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);

    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options
        .open(output)
        .map_err(|err| format!("failed to open {}: {err}", output.display()))?;

    #[cfg(unix)]
    {
        // mode() only applies to new files; --force may overwrite an existing one.
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|err| format!("failed to secure {}: {err}", output.display()))?;
    }

    file.write_all(content.as_bytes())
        .map_err(|err| format!("failed to write {}: {err}", output.display()))
}

fn random_hex(byte_count: usize) -> Result<String, String> {
    let mut bytes = vec![0_u8; byte_count];
    getrandom::fill(&mut bytes)
        .map_err(|err| format!("failed to read secure random bytes: {err}"))?;

    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }

    Ok(hex)
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn percent_encodes_connection_component_reserved_characters() {
        assert_eq!(
            percent_encode_connection_component("pa:ss@/word#1"),
            "pa%3Ass%40%2Fword%231"
        );
    }

    #[test]
    fn percent_encode_connection_component_keeps_unreserved_characters() {
        assert_eq!(
            percent_encode_connection_component("abcXYZ012-._~"),
            "abcXYZ012-._~"
        );
    }

    #[test]
    fn render_env_quotes_compose_values() {
        let mut config = Config::default();
        config.postgres_user = "user name".to_string();
        config.postgres_password = Some("pa$ss'word".to_string());
        config.postgres_db = "db#1".to_string();
        config.local_storage_dir = "/data/user files".to_string();
        config.cache_dir = "/cache\\files".to_string();

        let env = render_env(&mut config).unwrap();

        for expected in [
            "POSTGRES_USER='user name'",
            "POSTGRES_PASSWORD='pa$ss\\'word'",
            "POSTGRES_DB='db#1'",
            "DATABASE_URL='postgres://user%20name:pa%24ss%27word@postgres:5432/db%231'",
            "LOCAL_STORAGE_DIR='/data/user files'",
            "CACHE_DIR='/cache\\files'",
        ] {
            assert!(env.contains(expected), "missing {expected}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn write_output_uses_private_permissions() {
        for precreate in [false, true] {
            let output = std::env::temp_dir().join(format!(
                "hdrop-env-permissions-{}-{}",
                process::id(),
                random_hex(8).unwrap()
            ));

            if precreate {
                fs::write(&output, "old").unwrap();
                fs::set_permissions(&output, fs::Permissions::from_mode(0o644)).unwrap();
            }

            write_output(&output, "secret").unwrap();

            let mode = fs::metadata(&output).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
            fs::remove_file(output).unwrap();
        }
    }

    #[test]
    fn clap_help_includes_examples_and_options() {
        let help = Cli::command().render_long_help().to_string();

        assert!(help.contains("Generate a production Docker Compose .env file for hdrop."));
        assert!(help.contains("--storage-provider"));
        assert!(help.contains("cargo run -p hdrop-env -- --site hdrop.io"));
    }

    #[test]
    fn config_from_cli_recomputes_s3_defaults_from_region() {
        let cli = Cli::try_parse_from(["hdrop-env", "--s3-region", "us-east-1"]).unwrap();
        let config = Config::from_cli(cli);

        assert_eq!(config.s3_addressing_style, S3AddressingStyle::Path);
        assert_eq!(config.s3_region, "us-east-1");
        assert_eq!(config.s3_endpoint, "https://s3.us-east-1.amazonaws.com");
        assert_eq!(
            config.s3_public_url,
            "https://s3.us-east-1.amazonaws.com/hdrop"
        );
    }

    #[test]
    fn config_from_cli_keeps_explicit_s3_public_url() {
        let cli = Cli::try_parse_from([
            "hdrop-env",
            "--s3-endpoint",
            "https://objects.example.com",
            "--s3-bucket",
            "files",
            "--s3-public-url",
            "https://cdn.example.com/files",
        ])
        .unwrap();
        let config = Config::from_cli(cli);

        assert_eq!(config.s3_endpoint, "https://objects.example.com");
        assert_eq!(config.s3_bucket, "files");
        assert_eq!(config.s3_public_url, "https://cdn.example.com/files");
    }

    #[test]
    fn config_from_cli_supports_virtual_s3_addressing() {
        let cli = Cli::try_parse_from([
            "hdrop-env",
            "--storage-provider",
            "s3",
            "--s3-addressing-style",
            "virtual",
            "--s3-region",
            "us-east-1",
            "--s3-bucket",
            "files",
        ])
        .unwrap();
        let mut config = Config::from_cli(cli);

        assert_eq!(config.s3_addressing_style, S3AddressingStyle::Virtual);
        assert_eq!(
            config.s3_endpoint,
            "https://files.s3.us-east-1.amazonaws.com"
        );
        assert_eq!(
            config.s3_public_url,
            "https://files.s3.us-east-1.amazonaws.com"
        );
        assert!(render_env(&mut config)
            .unwrap()
            .contains("S3_ADDRESSING_STYLE=virtual"));
    }

    #[test]
    fn config_from_cli_supports_s3_request_timeout() {
        let cli = Cli::try_parse_from([
            "hdrop-env",
            "--storage-provider",
            "s3",
            "--s3-request-timeout-secs",
            "45",
        ])
        .unwrap();
        let mut config = Config::from_cli(cli);

        assert!(render_env(&mut config)
            .unwrap()
            .contains("S3_REQUEST_TIMEOUT_SECS=45"));
    }

    #[test]
    fn cli_rejects_invalid_s3_request_timeout() {
        assert!(Cli::try_parse_from(["hdrop-env", "--s3-request-timeout-secs", "0"]).is_err());
    }

    #[test]
    fn stdout_implies_non_interactive() {
        let cli = Cli::try_parse_from(["hdrop-env", "--stdout"]).unwrap();
        let config = Config::from_cli(cli);

        assert!(config.stdout);
        assert!(config.non_interactive);
    }
}
