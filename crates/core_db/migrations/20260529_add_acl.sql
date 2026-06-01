-- Add is_restricted to printers
ALTER TABLE printers ADD COLUMN is_restricted BOOLEAN NOT NULL DEFAULT 0;

-- Create printer_acl table
CREATE TABLE IF NOT EXISTS printer_acl (
    printer_id TEXT NOT NULL,
    entity_type TEXT NOT NULL, -- 'ou', 'user', 'ip'
    entity_name TEXT NOT NULL, -- The value to match (e.g. OU=Finance,DC=zeta,DC=local)
    FOREIGN KEY(printer_id) REFERENCES printers(id) ON DELETE CASCADE
);

CREATE INDEX idx_printer_acl_printer_id ON printer_acl(printer_id);
CREATE INDEX idx_printer_acl_entity ON printer_acl(entity_type, entity_name);
