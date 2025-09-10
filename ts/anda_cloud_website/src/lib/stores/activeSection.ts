import { writable } from 'svelte/store'
export const activeSection = writable<string>('')
export const sectionOrder = [
  'kip',
  'anda',
  'andadb',
  'andacloud',
  'get-started'
]
