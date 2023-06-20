import { AesBundle, DerivedKey } from './'

export default class DataBundle {
    constructor(public readonly derived_key: DerivedKey, public readonly aes_bundle: AesBundle) {}
}
