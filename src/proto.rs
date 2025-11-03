use std::collections::HashMap;
use extism_pdk::*;
use proto_pdk::*;

static NAME: &str = "ripgrep";

static REPO_URL: &str = "https://github.com/BurntSushi/ripgrep";

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::CommandLine,
        minimum_proto_version: Some(Version::new(0, 46, 0)),
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        ..RegisterToolOutput::default()
    }))
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_host_environment()?;

    check_supported_os_and_arch(
        NAME,
        &env,
        permutations![
            HostOS::Linux => [HostArch::X86, HostArch::X64, HostArch::Arm64, HostArch::Arm, HostArch::S390x],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Windows => [HostArch::X86, HostArch::X64, HostArch::Arm64],
        ],
    )?;

    let arch = match env.arch {
        HostArch::X86 => "i686",
        HostArch::X64 => "x86_64",
        HostArch::Arm64 => "aarch64",
        HostArch::Arm => "armv7",
        HostArch::S390x => "s390x",
        _ => unreachable!(),
    };

    let suffix = match (env.os, env.arch) {
        (HostOS::MacOS, _) => format!("{arch}-apple-darwin"),
        (HostOS::Windows, _) => format!("{arch}-pc-windows-msvc"),
        (HostOS::Linux, HostArch::X64) => format!("{arch}-unknown-linux-musl"),
        (HostOS::Linux, HostArch::Arm) => format!("{arch}-unknown-linux-musleabihf"),
        (HostOS::Linux, _) => format!("{arch}-unknown-linux-gnu"),
        _ => unreachable!(),
    };

    let archive_ext = match env.os {
        HostOS::Windows => "zip",
        _ => "tar.gz",
    };

    let version = &input.context.version;
    let filename_base = format!("ripgrep-{version}-{suffix}");

    Ok(Json(DownloadPrebuiltOutput {
        archive_prefix: Some(filename_base.clone()),
        checksum_url: Some(format!(
            "{REPO_URL}/releases/download/{version}/{filename_base}.{archive_ext}.sha256"
        )),
        download_url: format!(
            "{REPO_URL}/releases/download/{version}/{filename_base}.{archive_ext}"
        ),
        ..DownloadPrebuiltOutput::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;

    Ok(Json(LocateExecutablesOutput {
        exes: HashMap::from_iter([(
            "rg".into(),
            ExecutableConfig::new_primary(env.os.get_exe_name("rg")),
        )]),
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let regex = get_semver_regex();

    let tags = load_git_tags(REPO_URL)?
        .into_iter()
        .filter(|tag| regex.is_match(&tag))
        .collect::<Vec<_>>();

    Ok(Json(LoadVersionsOutput::from(tags)?))
}
