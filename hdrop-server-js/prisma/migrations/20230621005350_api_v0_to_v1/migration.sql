/*
  Warnings:

  - You are about to drop the column `fileNameHash` on the `File` table. All the data in the column will be lost.
  - Added the required column `challengeData` to the `File` table without a default value. This is not possible if the table is not empty.
  - Added the required column `challengeHash` to the `File` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "File" DROP COLUMN "fileNameHash",
ADD COLUMN     "challengeData" TEXT NOT NULL,
ADD COLUMN     "challengeHash" TEXT NOT NULL;
