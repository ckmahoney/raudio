/*
  Warnings:

  - You are about to drop the column `inputFile` on the `RenderRaw` table. All the data in the column will be lost.

*/
-- AlterTable
ALTER TABLE `RenderRaw` DROP COLUMN `inputFile`,
    ADD COLUMN `location` VARCHAR(191) NOT NULL DEFAULT '';
