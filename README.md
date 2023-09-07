# Nested VM Launcher
ネストされた仮想環境をlibvirt経由で作成し、その上でベンチマークを実行するためのスクリプトです。

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
```bash
./launcher.sh --l1-config ./example/l1-config.yaml --l2-config ./example/l2-config.yaml --bench-script ./example/run-bench.sh --output output.txt --dest dest
```

### How it works
`--dest`で指定されたディレクトリにVagrant用の設定ファイルを生成し、Vagrantを実行します。
`<dest>/l1-vagrant`がホストマシン上で実行されるL1 VM、`<dest>/l2-vagrant`がL1 VM上で実行されるL2 VM用のディレクトリです。
ただし、`l2-vagrant`の内容はL1 VMにコピーされ、L1 VM上で実行されるため、ホストマシン上では実行されず、ディレクトリ内の変更も反映されません。

`vagrant up`によってプロビジョニングが終了したあと、`--bench-script`で指定されたスクリプトをL2 VM上で実行します。
L2 VM上で実行されるスクリプトの標準出力結果は、`--output`で指定されたファイルに保存され、指定がなかった場合は標準出力に吐き出されます。
また、L1 VM上にも`/home/vagrant/bench-results.txt`という形で保存されます。

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


# FAQ

## Q. libvirtに指定するオプションを変更したい
### A. `cpus`などと同様にConfigurationのプロパティを追加実装する必要があります
`L1Vagrant`及び`L2Vagrant`が設定のyamlファイルを読み込むための構造体です。これらに対応するメンバーを追加する必要があります。
また、変更したyamlファイルをVagrantに読み込ませるために、`resouces/l1-vagrant-template/Vagrantfile`及び`resouces/l2-vagrant-template/Vagrantfile`を変更する必要があります。
追加されたプロパティをVagrantfileで読み取りlibvirtに渡してあげてください。
具体的にはすでに実装されている`cpus`や`memory`などを参考にしてください。
