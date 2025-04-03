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
          ];
        })
      ];
    });
  };
}
