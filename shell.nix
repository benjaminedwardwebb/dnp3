{ pkgs
, package
}:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
  ] ++ package.nativeBuildInputs;
}
