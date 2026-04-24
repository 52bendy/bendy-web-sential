-- Add avatar field to admin users table for profile picture support
ALTER TABLE bws_admin_users ADD COLUMN avatar TEXT;
