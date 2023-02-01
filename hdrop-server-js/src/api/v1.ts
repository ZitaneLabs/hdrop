import { Router } from "express"

import filesRouter from "./v1/filesRouter.js"

const router = Router()

router.use('/files', filesRouter)

export default router