import java.nio.file.Path;
import java.nio.file.Paths;

class Wormhole {

    static {
        Path p = Paths.get("target/debug/libmagic_wormhole_io_blocking.so");
        System.load(p.toAbsolutePath().toString());
    }

    private static native int receive(String server, String appid, String code);

    public static void main(String... args) {
        String s = "ws://127.0.0.1:4000/v1";
        String appId = "lothar.com/wormhole/text-or-file-xfer";
        
        System.out.println("received: " + Wormhole.receive(s, appId, "foobar"));
  }
}
