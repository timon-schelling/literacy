{
  inputs = {
    original.url = "github:timon-schelling/timonos";
  };

  outputs = { original, ... }: {
    nixosConfigurations.vm-dev = (original.nixosConfigurations.vm-dev-rust-leptos.extendModules {
      modules = [
        ({ pkgs, ...}: {
          environment.systemPackages = [
            pkgs.piper-tts
            pkgs.whisper-cpp
            pkgs.ffmpeg
            (pkgs.callPackage
              (
                { fetchFromGitHub, python3Packages }: python3Packages.buildPythonApplication {
                  pname = "whisperx";
                  version = "unstable-2025-04-04";
                  src = fetchFromGitHub {
                    owner = "m-bain";
                    repo = "whisperX";
                    rev = "f10dbf6ab1717e84db7733df9c0b21658ee68f9b";
                    hash = "sha256-LUsUqpQ/Cm2lIPaUE751j7px8/+rm6n6icde+SBlDEE=";
                  };
                  doCheck = false;
                  pyproject = true;
                  build-system = with python3Packages; [
                    setuptools
                  ];
                  dependencies = with python3Packages; [
                    ctranslate2
                    faster-whisper
                    nltk
                    numpy
                    onnxruntime
                    pandas
                    pyannote-audio
                    torch
                    torchaudio
                    transformers
                  ];
                  pythonRelaxDeps = [
                    "onnxruntime"
                    "torchaudio"
                  ];
                }
              )
              {}
            )
          ];
        })
      ];
    });
  };
}
