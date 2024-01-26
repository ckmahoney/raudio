-- AlterTable
ALTER TABLE `Recording` MODIFY `name` VARCHAR(191) NULL DEFAULT '';

-- AlterTable
ALTER TABLE `RenderTask` ADD COLUMN `name` VARCHAR(191) NOT NULL DEFAULT '';
