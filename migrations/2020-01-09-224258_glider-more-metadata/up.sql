ALTER TABLE gliders
    -- When was the glider acquired?
    ADD COLUMN since DATE,
    -- When was the glider sold / given away / thrown away?
    ADD COLUMN until DATE,
    -- Where did you get the glider from? (e.g. a shop, or a website)
    ADD COLUMN source TEXT,
    -- How much did the glider cost, in your currency?
    ADD COLUMN cost INTEGER,
    -- Add arbitrary comments about this glider
    ADD COLUMN comment TEXT;
