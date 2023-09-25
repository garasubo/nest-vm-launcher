# Nested VM Launcher
ネストされた仮想環境をlibvirt経由で作成し、その上でベンチマークを実行するためのスクリプトです。

## Features
- L1 VM及びL2 VMの作成
- L2 VM上でのベンチマーク実行
    - `sysbench`と`phronix-test-suite`が利用できます
- L2 VM相当の環境をL1 VMとして作成してベンチマーク実行
    - ネストされた環境とそうでない環境との比較に便利な機能です

## Requirements
以下のツール・パッケージが必要です。
- Vagrant
- Vagrant Libvirt Plugin
- Rust
- rsync

## Usage
`launcher.sh`を経由して実行します。このシェルスクリプト自体はどこから実行しても問題ありませんが、このシェルスクリプトそのものを別ディレクトリに移動することはできません。
`./launcher.sh --help`でヘルプを表示します。

### 実行例
`./example`以下にあるファイルを使い実行する例です。

新しく作業用のディレクトリを作成し、VMをつくる場合は以下のように実行します。
```bash
./launcher.sh create --l1-config ./example/l1-config.yaml --l2-config ./example/l2-config.yaml --bench-script ./example/run-bench.sh --output output.txt --project-dir dest
```

既存のディレクトリを利用して作成済みのVMを起動する場合は以下のように実行します。
```bash
./launcher.sh provision --l1-config ./example/l1-config.yaml --l2-config ./example/l2-config.yaml --bench-script ./example/run-bench.sh --output output.txt --project-dir dest
```

作成済みのVM上でベンチマーク実行を行う場合は以下のように実行します。
```bash
./launcher.sh run-bench --bench-script ./example/run-bench.sh --output output.txt --project-dir dest
```

L2 VMをL1 VMとして使う場合は`--no-nested`オプションをつけてください。この場合、利用できるオプションが通常のL2 VMとは異なることに注意してください。
```bash
./launcher.sh create --project-dir ./dest --l2-config example/l2-no-nested-config.yaml --bench-script example/run-bench.sh --output output.txt  --no-nested
```

### How it works
`--project-dir`で指定されたディレクトリにVagrant用の設定ファイルを生成し、Vagrantを実行します。
`<project-dir>/l1-vagrant`がホストマシン上で実行されるL1 VM、`<project-dir>/l2-vagrant`がL1 VM上で実行されるL2 VM用のディレクトリです。
`l2-vagrant`の内容はL1 VMのsync folderとして設定されるので、`vagrant reload`コマンドなどによりホストマシンでの変更がL1 VMに反映されます。
`--no-nested`を使った場合は`<project-dir>/l2-vagrant-no-nested`がL2 VM用のディレクトリとなります。

`vagrant up`によってプロビジョニングが終了したあと、`--bench-script`で指定されたスクリプトをL2 VM上で実行します。
L2 VM上で実行されるスクリプトの標準出力結果は、`--output`で指定されたファイルに保存され、指定がなかった場合は標準出力に吐き出されます。
また、L1 VM上にも`/home/vagrant/bench-results.txt`という形で保存されます。

L2 VMには`sysbench`及び`phoronix-test-suite`がインストールされています。
他にも必要なパッケージがある場合はベンチマークスクリプトで適宜インストールしてください。

## Configuration
L1 VM及びL2 VMの設定はyamlファイルで行います。それぞれ以下のようなプロパティが利用できます。

### L1 VM
- `host_name`: L1 VMのホスト名
- `memory`: L1 VMのメモリサイズ(MB)
- `cpus`: L1 VMのCPUコア数
- `cpu_mode`: L1 VMのCPUモード
- `network_interface`: L1 VMのブリッジ接続に使うネットワークインターフェース。指定しない場合はブリッジ接続を行わない。

### L2 VM
- `host_name`: L2 VMのホスト名
- `memory`: L2 VMのメモリサイズ(MB)
- `cpus`: L2 VMのCPUコア数
- `cpu_mode`: L2 VMのCPUモード
- `enable_network_bridge`: L2 VMのブリッジ接続を有効にするかどうか

### L2 VM (no nested)
- `host_name`: L2 VMのホスト名
- `memory`: L2 VMのメモリサイズ(MB)
- `cpus`: L2 VMのCPUコア数
- `cpu_mode`: L2 VMのCPUモード
- `network_interface`: L2 VMのブリッジ接続に使うネットワークインターフェース。指定しない場合はブリッジ接続を行わない。

# FAQ

## Q. libvirtに指定するオプションを変更したい
### A. `cpus`などと同様にConfigurationのプロパティを追加実装する必要があります
`L1Vagrant`及び`L2Vagrant`が設定のyamlファイルを読み込むための構造体です。これらに対応するメンバーを追加する必要があります。
また、変更したyamlファイルをVagrantに読み込ませるために、`resouces/l1-vagrant-template/Vagrantfile`及び`resouces/l2-vagrant-template/Vagrantfile`を変更する必要があります。
追加されたプロパティをVagrantfileで読み取りlibvirtに渡してあげてください。
具体的にはすでに実装されている`cpus`や`memory`などを参考にしてください。
