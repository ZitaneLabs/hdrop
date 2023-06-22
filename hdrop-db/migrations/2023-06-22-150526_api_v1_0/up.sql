-- Your SQL goes here
ALTER TABLE "files" DROP COLUMN "fileNameHash",
ADD COLUMN     "challengeData" TEXT NOT NULL,
ADD COLUMN     "challengeHash" TEXT NOT NULL;
