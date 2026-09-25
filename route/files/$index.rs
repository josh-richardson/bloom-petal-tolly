petal::route_file!(
    spec: petal::static_dir_spec(),
    list: {
        let mut children = petal::files(&["README.md", "AGENTS.md", "status.json", "markets.json", "positions.json"]);
        children.extend(petal::dir_names(&["tokens", "quote", "operations"]));
        children.extend(petal::files(&["buy.json", "sell.json", "launch.json"]));
        children
    }
);
