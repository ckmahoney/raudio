/*
  Warnings:

  - Added the required column `format` to the `Recording` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE `Recording` ADD COLUMN `format` ENUM('mp3', 'aiff') NOT NULL;
