const std = @import("std");
const builtin = @import("builtin");
const clap = @import("clap");

const debug = std.debug;
const io = std.io;

pub fn fatal(comptime format: []const u8, args: anytype) noreturn {
    std.log.err(format, args);
    std.process.exit(1);
}

pub fn fetch_orders(token: []const u8, start: []const u8, end: []const u8) noreturn {
    const url_base = "https://typeractive.myshopify.com";
    const url_endpoint = "/admin/api/2021-07/orders.json";
    const url_params = std.fmt.allocPrint(std.heap.page_allocator, "?status=any&fields=line_items,financial_status,current_subtotal_price&created_at_min={s}&created_at_max={s}", .{ start, end });
    defer std.heap.page_allocator.free(url_params);
    const url = std.fmt.allocPrint(std.heap.page_allocator, "{s}{s}{s}", .{ url_base, url_endpoint, url_params });
    defer std.heap.page_allocator.free(url);

    const parsed_url = try std.Uri.parse(url);
    const headers = std.http.Headers{ .allocator = std.heap.page_allocator };
    defer headers.deinit();

    try headers.append("X-Shopify-Access-Token", token);

    const client: std.http.Client = .{ .allocator = std.heap.page_allocator };
    defer client.deinit();

    const request = try client.request(.GET, parsed_url, headers, .{});

    try request.start();
    try request.do();

    const result = request.reader().readAllAlloc(std.heap.page_allocator);
    defer std.heap.page_allocator.free(result);

    std.debug.print("{}", .{ result });
}

pub fn main() !void {
    const params = comptime clap.parseParamsComptime(
        \\-h, --help             Display this help and exit.
        \\-m, --month <u32>      Month
        \\-y, --year <i32>       Year
        \\-t, --token <string>   Shopify Token
        \\
    );

    var diag = clap.Diagnostic{};
    var res = clap.parse(clap.Help, &params, clap.parsers.default, .{
        .diagnostic = &diag,
    }) catch |err| {
        diag.report(io.getStdErr().writer(), err) catch {};
        return err;
    };
    defer res.deinit();

    if (res.args.help != 0)
        return clap.help(std.io.getStdErr().writer(), clap.Help, &params, .{});

    const month = res.args.month orelse fatal("Missing argument `--month`", .{});
    const year = res.args.year orelse fatal("Missing argument `--year`", .{});

    const month_end = if (month == 12) 1 else month + 1;
    _ = month_end;
    const year_end = if (month == 12) year + 1 else year;
    _ = year_end;

    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();

    const env_map = try arena.allocator().create(std.process.EnvMap);
    env_map.* = try std.process.getEnvMap(arena.allocator());
    defer env_map.deinit();

    const token = (if (res.args.token != null) res.args.token else env_map.get("SHOPIFY_TOKEN")) orelse fatal("Missing argument `--token`", .{});

    fetch_orders(token, "test", "test");
}
