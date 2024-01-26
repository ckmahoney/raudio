/*
  Warnings:

  - You are about to alter the column `label` on the `Recording` table. The data in that column could be lost. The data in that column will be cast from `Enum(EnumId(1))` to `VarChar(191)`.

*/
-- AlterTable
ALTER TABLE `Recording` MODIFY `label` VARCHAR(191) NOT NULL;
