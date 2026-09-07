-- This file should undo anything in `up.sql`
ALTER TABLE "files" DROP COLUMN "challengeData", DROP COLUMN "challengeHash", ADD COLUMN "fileNameHash" TEXT NOT NULL;
