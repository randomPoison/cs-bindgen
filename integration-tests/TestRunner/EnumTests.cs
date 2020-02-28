using System;
using System.Linq;
using Xunit;

namespace TestRunner
{
    public class EnumTests
    {
        [Fact]
        public void SimpleEnumRoundTrip()
        {
            foreach (var variant in Enum.GetValues(typeof(SimpleCEnum)).Cast<SimpleCEnum>())
            {
                SimpleCEnum result = IntegrationTests.RoundTripSimpleCEnum(variant);
                Assert.Equal(variant, result);
            }
        }
    }
}
