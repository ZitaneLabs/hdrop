import { Router } from "express"
import Prometheus from 'prom-client'

import filesRouter from "./v1/filesRouter.js"

const router = Router()

router.use('/files', filesRouter)
router.use('/metrics', async (_req, res) => {
    try {
        res.set('Content-Type', Prometheus.register.contentType)
        res.end(await Prometheus.register.metrics())
    } catch (err) {
        res.status(500).end(err)
    }
})

export default router