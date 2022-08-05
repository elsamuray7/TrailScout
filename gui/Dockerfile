# Stage 1: Compile and Build angular codebase

# Use official node image as the base image
FROM node:latest as build

# Set the working directory
WORKDIR /usr/local/app

# Add the source code to app
COPY ./ /usr/local/app/

# Install all the dependencies
RUN npm install

# Generate the build of the application
# Depends on the build-arg ENV
# Possible values in gui/package.json "scripts"
# We use either build or dev

ARG ENV
RUN npm run $ENV

# Stage 2: Serve app with nginx server

# Use official nginx image as the base image
FROM nginx:latest

# Copy the build output to replace the default nginx contents.
COPY --from=build /usr/local/app/dist/gui /usr/share/nginx/html

# Expose port 80
EXPOSE 80
