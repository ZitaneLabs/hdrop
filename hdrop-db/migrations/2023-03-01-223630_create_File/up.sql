CREATE TABLE IF NOT EXISTS "files" (
            "uuid" UUID NOT NULL,
            "accessToken" TEXT NOT NULL,
            "updateToken" TEXT NOT NULL,
            "dataUrl" TEXT,
            "fileNameData" TEXT NOT NULL,
            "fileNameHash" TEXT NOT NULL,
            "salt" TEXT NOT NULL,
            "iv" TEXT NOT NULL,
            "createdAt" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "expiresAt" timestamp with time zone NOT NULL,
        
            CONSTRAINT "files_pkey" PRIMARY KEY ("uuid")
        );

        CREATE UNIQUE INDEX "files_uuid_key" ON "files"("uuid");
        CREATE UNIQUE INDEX "files_accessToken_key" ON "files"("accessToken");