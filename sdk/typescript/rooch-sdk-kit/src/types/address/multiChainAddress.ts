// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getAddressInfo } from './bitcoinAddress'
import { bech32 } from 'bech32'
import { RoochMultiChainID, RoochMultiChainIDToString } from '@roochnetwork/rooch-sdk'
import { bcs } from '@roochnetwork/rooch-sdk'

export class MultiChainAddress implements bcs.Serializable {
  private readonly multiChainId: RoochMultiChainID
  private readonly rawAddress: Uint8Array

  // TODO: support all Chain
  constructor(multiChainId: RoochMultiChainID, address: string) {
    this.multiChainId = multiChainId

    this.rawAddress = getAddressInfo(address).bytes
  }

  toBytes(): Uint8Array {
    let bs = new bcs.BcsSerializer()
    this.serialize(bs)
    return bs.getBytes()
  }

  // TODO: remove this, add toString
  toBech32(): string {
    const data = [1].concat(bech32.toWords(this.rawAddress))
    const address = bech32.encode(RoochMultiChainIDToString(this.multiChainId), data)
    return address
  }

  serialize(se: bcs.BcsSerializer) {
    se.serializeU64(this.multiChainId)
    bcs.Helpers.serializeVectorU8(Array.from(this.rawAddress), se)
  }
}
