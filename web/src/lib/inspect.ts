import { createContext, useContext } from 'react'
import type { Service } from './types'

// What the drawer is showing: a specific port's services (0..n) or a single
// row. `port` is set only for a lookup, so an empty result can say which.
export interface InspectTarget {
  port?: number
  services: Service[]
}

export const InspectContext = createContext<(target: InspectTarget) => void>(() => {})

export function useInspect() {
  return useContext(InspectContext)
}
