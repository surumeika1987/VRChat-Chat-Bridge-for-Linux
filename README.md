# VRChat Chat Bridge for Linux
Linux上での日本語のチャットを可能にします。
![Image](https://github.com/user-attachments/assets/e1802e54-0590-4886-a1b8-d6a91165bbb5)

## 使用方法
vrc_chat_bridgeを起動します。このプログラムはIPC通信を利用しています。  
IPC通信を行うにはプログラムに追加の引数を指定してください。  
あらかじめ本プログラムを引数なしで起動しておいてください。  

| 引数 | 説明 |
| ---- | ---- |
| toggle | GUIの表示、非表示を切り替えます |
| show | GUIを表示します |
| hide | GUIを非表示にします |

## ビルド方法
Rustがインストールされていることを確認します。
cargoを利用してビルドを行います。
```shell
$ cargo build --release
```
`target/release/vrc_chat_bridge`がプログラム本体です。

## ライセンス
MIT
