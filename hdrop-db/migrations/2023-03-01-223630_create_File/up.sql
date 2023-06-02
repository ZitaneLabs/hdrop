CREATE TABLE IF NOT EXISTS "files" (
            "id" SERIAL NOT NULL,
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
        
            -- CONSTRAINT "File_pkey" PRIMARY KEY ("id"),
            CONSTRAINT "files_pkey" PRIMARY KEY ("uuid")
        );

        CREATE UNIQUE INDEX "files_uuid_key" ON "files"("uuid");
        CREATE UNIQUE INDEX "files_accessToken_key" ON "files"("accessToken");



CREATE TABLE IF NOT EXISTS "statistics" (
            "id" SERIAL NOT NULL,
            "uploadCount" INTEGER NOT NULL DEFAULT 0,
            "deleteCount" INTEGER NOT NULL DEFAULT 0,
            "existCount"  INTEGER NOT NULL DEFAULT 0,

            CONSTRAINT "statistics_pkey" PRIMARY KEY ("id")
        );