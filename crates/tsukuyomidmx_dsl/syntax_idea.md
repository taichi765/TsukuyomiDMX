```
// importで他のファイルからmodule(ないしfn)をインポートできる。
// ESModuleやSlintに近い。
import { blink } from "../blink.tkd"

// .xxxでフィクスチャについたタグのクエリ。||や&&でAND/ORができたらいいが構文は未定
red_blink().bind_to(".front")

// moduleかfnかどちらにするかは未定
module red_blink(){
    // イミュータブル
    blink().color("#FF0000")
}

module scene1(){
    // hex以外も使える
    // #で名前指定
    parallel(
        dimmer(255).color(1.0, 0.5, 0.5)
            .bind_to("#adj-pocket-pro-1")
            .gobo("gobo1"), //fixture-specificな操作はbind_toでFixtureのコンテキストを付加してから
        dimmer(100).color(0.5, 0.5, 1.0)
            .bind_to("#adj-pocket-pro-2")
    )
}

sequence(
    step(200, scene1()),
    fadein(50),
    step(100, parallel(
        scene2_front(),
        scene2_back(),
    )),
)
```
