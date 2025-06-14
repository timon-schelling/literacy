{
  inputs = {
    original.url = "github:timon-schelling/timonos";
  };

  outputs = { original, ... }: {
    nixosConfigurations.vm-dev = (original.nixosConfigurations.vm-dev-rust-leptos.extendModules {
      modules = [
        ({ pkgs, lib, ...}: {
          environment.systemPackages = [
            pkgs.piper-tts
            pkgs.whisper-cpp
            pkgs.ffmpeg
            (pkgs.callPackage
              ({ fetchFromGitHub, python3Packages }: python3Packages.buildPythonApplication {
                pname = "ctc-forced-aligner";
                version = "unstable-2025-04-06";
                src = fetchFromGitHub {
                  owner = "MahmoudAshraf97";
                  repo = "ctc-forced-aligner";
                  rev = "201276a4ea2ddd3f5caead1ac4f211477ae3da6d";
                  hash = "sha256-xReCGuOq3o/PawhFlkTW3BobMLR5tvucgUNY8crIOZQ=";
                };
                doCheck = false;
                pyproject = true;
                build-system = with python3Packages; [
                  setuptools
                ];
                dependencies = with python3Packages; [
                  nltk
                  torch
                  torchaudio
                  transformers
                  unidecode
                ];
              })
              {}
            )
            (pkgs.callPackage
              ({ fetchFromGitHub, python3Packages }: python3Packages.buildPythonApplication {
                pname = "uroman";
                version = "unstable-2025-04-06";
                src = fetchFromGitHub {
                  owner = "isi-nlp";
                  repo = "uroman";
                  rev = "86a196e363e98df2ba0e86b4ea690676519817f2";
                  hash = "sha256-fusCo25gc2UCExAhU/UR+veoAw8y4UKza3l35FqPsnI=";
                };
                doCheck = false;
                pyproject = true;
                build-system = with python3Packages; [
                  setuptools
                ];
                dependencies = with python3Packages; [
                  regex
                  hatchling
                ];
              })
              {}
            )
          ];
          contain.config.filesystem.disks = [
            {
              source = "target/vm.disk.qcow2";
              tag = "target";
              size = 30000;
            }
          ];

          systemd.services.own-target-dir = {
            wantedBy = [ "multi-user.target" ];
            after = [ "local-fs.target" ];
            serviceConfig = {
              Type = "oneshot";
              RemainAfterExit = true;
              ExecStart = ''
                ${pkgs.coreutils}/bin/chown user:users /home/user/target
              '';
            };
          };
          fileSystems."/home/user/target" = {
            device = "/dev/disk/by-id/virtio-target";
            fsType = "btrfs";
            neededForBoot = true;
            autoFormat = true;
            options = [
              "x-initrd.mount"
              "defaults"
              "x-systemd.requires=systemd-modules-load.service"
            ];
          };
          home-manager.users.user.programs.nushell.extraConfig = lib.mkAfter ''
            $env.CARGO_TARGET_DIR = $"($env.HOME)/target"
          '';
        })
      ];
    });
  };
}
