-- Migration 005: Add hosting_service column to bws_domains
ALTER TABLE bws_domains ADD COLUMN hosting_service TEXT DEFAULT NULL;