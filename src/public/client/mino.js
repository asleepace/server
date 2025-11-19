// Placeholder module – ensure wc runtime auto-bootstrap is available to pages
import { bootstrapWCAuto } from './wc/runtime.js'
try { bootstrapWCAuto() } catch { }

export function elem() { }