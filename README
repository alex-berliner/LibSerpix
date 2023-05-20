### LibSerpix
A pure-lua addon allowing realtime transmission of data out of the World of Warcraft client by serializing data as pixels and displaying it on the screen.

### Download

### Usage
1. Copy LibSerpix to your project's libraries folder
2. Include `LibSerpix\LibSerpix.xml` in your .toc
3. Use `LibSerpix = LibStub("LibSerpix-0.1")` or equivalent to get a handle to the library
4. Decide a namespace for your addon. The addon is single user right now but if I ever figure out how to accept data from multiple users this will be important :)
5. Create a table containing your message.
```
local data = {}
data["my_message"] = "message contents"
```
6. Send your message. LibSerpix will automatically transmit it over the screen!
```
LibSerpix.add_data("namespace", data)
```
![Pixel look](assets/pixels.png "Pixel look")
### Spec
LibSerpix transmits data by drawing a series of pixel columns to the to the top-left of the screen. LibSerpix expects that an external program is continuously reading the screen to process messages. The communication is not bidirectional; there is no mechanism for the realtime transmission of data back into the WoW client.

A WoW pixel has 4 channels: RGBA, each with 256 values (aka 1 byte). We have to chop off the alpha channel because setting it to anything besides maximum would compromise the perceived value of the other channels, so we're left with being able to encode 3 bytes of data per pixel to RGB. Thus we can draw arbitary byte data to the screen as by encoding it as a series of pixels.

LibSerpix takes some liberties with the size of its drawn output. Instead of 1x1 pixel rows with no spaces, it draws 1 pixel wide columns of pixels and draws them spaced apart with 1 pixel. To explain:

1. WoW may sometimes draw the pixel rows 2 pixels wide instead of 1. If the pixels had no spaces in between then pixels would often get overlapped and screw up transmission of the whole frame.
2. When processing the screen capture, the pixels are sometimes caught between 2 states. Using columns of pixels instead of 1x1's gives a higher probability that the correct pixel will be captured. For each column, the pixel with the highest occurrence should be considered the real one.

## Frame Format
![Frame format](assets/frameformat.png "Frame format")

The first column is always the anchor (#2a0045): its purpose is to allow screen reader programs to reliably find the column array.

The second column is the header. It only contains enough information to allow the program to decode the data section. The size is the size of the Data section in bytes. The checksum is a basic 8-bit checksum over the Data section (sum of the Data section bytes modulo 256).

The remaining bytes are the Data, which is a single CBOR message. This can be of any size though the library allocates 512 pixels by default (modify `PAYLOAD_BYTES` to increase).

## Columns
Each column starts and ends with the anchor color (#2a0045) as a hint to the screen reader as to which columns of the screenshot contain pixel data. By default columns are 8 pixels high; the pixel data takes up 6 pixels and the anchor takes up 2 at the top and bottom. The recommended way to search for columns containing pixel data is to:

1. Find the anchor column containing all #2a0045 by scanning from the left to the right
2. Look for the header column by looking for the next colum that has the anchor color at the top and bottom. Parse the header column (next after anchor) to determine how many pixels are in the Data segment
3. Look for the remaining header-indicated number of pixels using the same scanning method as in (2)
