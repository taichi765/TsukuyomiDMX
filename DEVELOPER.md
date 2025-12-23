## Coding Guidelines
- Avoid using `in-out property` if there's another way.
- Rust code and UI communicate via global singletons named `XxxAdaoter`.
- Small components should not access Adapters directory. Use `in property` instead.

- データの流れ：ユーザーアクション -> Adapterのコールバック -> update(Msg)で状態を更新 -> 派生状態も更新(自前で実装するかSlintのMapModelを使うかは未定)。
- Observerを雑に使わない。データの流れを見づらくなるため。SlintのModelがやるように、型の内部に隠蔽する。
- コンポーネント内でしか使わないローカルな状態はSlint内で完結させる。
- Rust側で持つ状態を使いたい場合はAdapterのin propertyを使う。
- Adapterでout propertyは使わない。
- Adapterのcallbackはメッセージとしてのみ使う。バインドしたい場合はコールバックの返り値をバインドするのではなく、コールバックがin propertyを更新するようにする(Adapterのcallbackが純粋であることは誰も保証してくれないため)。
- コンポーネント間で共有される状態はすべてAppStateにまとめる(Redux・TEA的アプローチ)。状態の更新はupdateを介してのみ行う。
- 

## Directory structure
- `containers/` are components to handle Store and pass properties to children views.
- `view/` are components to represent object.
- `parts/` are small and reusable components.
