import { useOutletContext } from 'react-router-dom'
import type { ResourceOutletContext } from '@/pages/ResourceLayout'

export function useResourceDef() {
  return useOutletContext<ResourceOutletContext>().def
}
