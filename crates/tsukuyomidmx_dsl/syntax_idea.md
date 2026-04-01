```
// importで他のファイルからmodule(ないしfn)をインポートできる。
// ESModuleやSlintに近い。
import { blink } from "../blink.tkd"

// .xxxでタグのクエリ。||や&&でAND/ORができたらいいが構文は未定
red_blink().apply(".front")

// moduleかfnかどちらにするかは未定
module red_blink(){
    // イミュータブル
    blink().color("#FF0000")
}
```