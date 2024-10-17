export def pack [archive: string, ...files: string] {
    if (sys host).name == "Windows" {
        let command = $"Compress-Archive -Path ($files | str join ', ') -DestinationPath ($archive)"

        if (which pwsh | is-empty) {
            powershell -Command $command
        } else {
            pwsh -Command $command
        }
    } else {
        ^zip -r $archive ...$files
    }
}

export def dist [...targets: string] {
    let version = (open Cargo.toml).package.version

    for target in $targets {
        let family = if ($target | str contains "darwin") {
            'darwin'
        } else if ($target | str contains "windows") {
            'windows'
        } else {
            continue
        }

        mut dy: any = null
        match $family {
            'darwin' => {
                $dy = [libuvc_control.dylib]
            }
            'windows' => {
                $dy = [uvc_control.dll uvc_control.dll.lib]
            }
        }

        rustup target add $target
        cargo build --target $target --release
        for name in $dy {
            cp $"target/($target)/release/($name)" $"./($name)"
        }
        pack $"target/uvc-control-v($version)-($target).zip" include ...$dy
        rm ...$dy

        match $family {
            'darwin' => {
                pack $"target/libuvc_control.dylib.dSYM-v($version)-($target).zip" $"target/($target)/release/libuvc_control.dylib.dSYM"
            }
            'windows' => {
                cp $"target/($target)/release/uvc_control.pdb" $"target/uvc_control-v($version)-($target).pdb"
            }
        }
    }
}
