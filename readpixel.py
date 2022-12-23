import time
import pyautogui
import pywinauto

def colorToInteger(color):
    # Unpack the RGB values
    r, g, b = color

    # Convert the RGB values to integers in the range 0-255
    r = int(r)
    g = int(g)
    b = int(b)

    # Encode the integers as a single number
    return r * 256 * 256 + g * 256 + b

# Define a function to check the active window's title
def check_window_title(title):
    # Get the window handles for windows with the specified title
    windows = pywinauto.findwindows.find_windows(title=title)
    # Check if there are any windows with the specified title
    if windows:
        # Return True if there are windows with the specified title
        return True
    else:
        # Return False if there are no windows with the specified title
        return False

# Run the loop indefinitely
v = -1
while True:
    # Check if the active window's title is "World of Warcraft"
    # if check_window_title("World of Warcraft"):
        # Get the top left pixel of the active window
    pixel = pyautogui.pixel(0, 0)
    # print(pixel)
    # Output the pixel values
    vnew = colorToInteger(pixel)
    if vnew != v:
        print(vnew, pixel)
        v = vnew
    # else:
    #     # Output a message if the active window is not "World of Warcraft"
    #     print("Active window is not World of Warcraft")
    # time.sleep(0.05)
