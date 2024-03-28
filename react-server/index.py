from flask import Flask, Response
import cv2
app = Flask(__name__)

@app.route('/stream', methods=['GET'])
def stream():
    return Response(generate(), mimetype='multipart/x-mixed-replace; boundary=frame')

def generate():
    #initialize the camera
    vc = cv2.VideoCapture(0)

    #check if camera is open
    if vc.isOpened():
        rval, frame = vc.read()
    else:
        rval = False
    
    #while streaming
    while rval:
        #encode the frame in JPEG format
        rval, frame = vc.read()
        if frame is None:
            continue

        #encode the frame in JPEG format
        (flag, encodedImage) = cv2.imencode(".jpg", frame)
        
        #ensure the frame was successfully encoded
        if not flag:
            continue

        #yield the output frame in the byte format
        yield (b'--frame\r\n' b'Content-Type: image/jpeg\r\n\r\n' +
            bytearray(encodedImage) + b'\r\n')
        
    vc.release()

if __name__ == '__main__':
    host = "127.0.0.1"
    port = 5000
    app.run(host=host, port=port)