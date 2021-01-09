#ifndef MIDDLEWARE_H_INCL
#define MIDDLEWARE_H_INCL

#include <filesystem>
#include <map>
#include <cstring>
#include <fstream>
#include <sstream>
#include <iostream>
#include <future>
#include <string_view>
#include <uWebSockets/App.h>

// Middleware from uWebSocket examples

/* Middleware to fill out content-type */
inline bool hasExt(std::string_view file, std::string_view ext) {
    if (ext.size() > file.size()) {
        return false;
    }
    return std::equal(ext.rbegin(), ext.rend(), file.rbegin());
}

/* This should be a filter / middleware like app.use(handler) */
template <bool SSL>
uWS::HttpResponse<SSL> *serveFile(uWS::HttpResponse<SSL> *res, uWS::HttpRequest *req) {
    res->writeStatus(uWS::HTTP_200_OK);

    if (hasExt(req->getUrl(), ".svg")) {
        res->writeHeader("Content-Type", "image/svg+xml");
    }

    return res;
}

#endif