// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { useStore } from 'zustand'

import { WalletContext } from '../../provider/walletProvider'
import type { WalletStoreState } from '../../walletStore'

export function useWalletStore<T>(selector: (state: WalletStoreState) => T): T {
  const store = useContext(WalletContext)
  if (!store) {
    throw new Error('Could not find WalletContext. Ensure that you have set up the WalletProvider.')
  }
  return useStore(store, selector)
}
